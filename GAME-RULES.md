# Poker Game Rules Reference

A concise rules reference for each game variant supported by the poker simulator. This document defines the exact rules the evaluator engine must implement.

---

## Simulation Interface

The simulator supports two **simulation types** (see `config/games.toml`):

| Type | Games | Held cards | Behavior |
|------|-------|------------|----------|
| **Draw** | 2-7, A-5 | 0–5 | User holds cards they're keeping. Simulate random draws to complete hand. Each result = completed 5-card hand. Percentile = where hand ranks vs. all possible draws. |
| **Board** | Hold'em, Omaha, O8 | 0–2 (Hold'em) or 0–4 (Omaha) | User holds hole cards. Simulate random boards (5 community cards). Pad hole with random when held &lt; max. Each result = the board (never contains hole cards). Percentile = where hero's hand ranks vs. all possible boards. |

**Dead cards** are always excluded from the simulation pool (folded cards, opponent mucks, etc.).

To add a new game: implement the evaluator, add to `games.toml` with `simulation_type`, and extend `GameVariant` in poker-eval.

---

## NL 2-7 Single Draw (Deuce-to-Seven Lowball)

**Goal:** Make the lowest possible five-card hand. This is inverse poker — the worst traditional poker hand wins.

**Ace Treatment:** Aces are always high. An ace is the worst card you can hold. A-5-4-3-2 is not a straight — it's an ace-high hand (and a terrible one).

**Straights:** Count against you. If your five cards form a straight, it's a bad hand. Holding 6-5-4-3-2 is a straight, not a six-low.

**Flushes:** Count against you. If all five cards share a suit, it's a flush and hurts your hand.

**Pairs/Trips/etc:** Count against you. Any paired cards are bad, same as traditional poker but inverted.

**Best Hand:** 7-5-4-3-2 (offsuit, at least two suits represented). Known as "the nuts," "number one," or "the wheel" in 2-7 context.

**Second Best:** 7-6-4-3-2 offsuit.

**Third Best:** 7-6-5-3-2 offsuit.

**Worst Possible Hand:** A royal flush (A-K-Q-J-T all same suit).

**Hand Ranking Logic:** Compare hands from the highest card down. Lower is better. A 9-8-7-6-4 beats J-5-4-3-2 because 9-high beats J-high. If the highest cards tie, compare the second highest, and so on.

**Street Structure (2 streets):**

1. **Pre-Draw:** Each player is dealt 5 cards face down. Blinds are posted (small blind, big blind). A round of betting occurs starting left of the big blind. Players may fold, call, or raise. No limit betting — players can bet any amount up to their full stack.
2. **Draw:** Each remaining player may discard 0 to 5 cards and receive replacements from the deck. Discarding zero cards is called "standing pat." Discarding is done one player at a time, starting left of the dealer.
3. **Post-Draw:** A second round of betting occurs. After betting concludes, remaining players show down. Lowest hand wins.

**Betting Structure:** Typically no-limit. Often played with an ante in addition to blinds to stimulate action.

**Common Gotchas for the Evaluator:**

- A-2-3-4-5 is NOT a straight (ace is high, so it's A-5-4-3-2, an ace-high hand — still awful, but not a straight).
- 2-3-4-5-6 IS a straight (bad).
- Must check for flushes — if all five cards share a suit, that's a flush (bad).
- 8-6-5-4-3 is commonly called a "smooth eight." 8-7-6-5-4 would be a straight (bad).

---

## Ace-to-Five Lowball (A-5) / Razz

These are two different game formats that share the same hand ranking system. A-5 Draw is a draw game; Razz is a stud game. The evaluator logic is identical for both.

**Goal:** Make the lowest possible five-card hand.

**Ace Treatment:** Aces are always low. Ace is the best card you can hold.

**Straights:** Do NOT count against you. They are completely ignored. A-2-3-4-5 is simply a five-low, not a straight.

**Flushes:** Do NOT count against you. They are completely ignored. Even A-2-3-4-5 all of the same suit is still the best hand.

**Pairs/Trips/etc:** Count against you, same as 2-7. Any no-pair hand beats any hand with a pair.

**Best Hand:** A-2-3-4-5 (the "wheel" or "bicycle"). This is the nuts regardless of suits.

**Second Best:** A-2-3-4-6 (six-four low).

**Worst Possible Hand:** K-K-K-K-Q (four kings with a queen kicker — or any quads/full house; effectively the standard poker rankings inverted but without straights/flushes counting).

**Hand Ranking Logic:** Compare from highest card down, lower is better. 7-5-4-3-2 beats 7-6-3-2-A because at the second card, 5 < 6. Pairs are always worse than no-pair hands. Among paired hands, lower pairs are better (pair of aces beats pair of deuces since ace is low).

### A-5 Triple Draw (Draw Format)

**Street Structure (4 betting rounds, 3 draws):**

1. **Pre-Draw:** Five cards dealt face down. Blinds posted. First betting round.
2. **First Draw:** Players discard 0–5 cards and draw replacements. Second betting round.
3. **Second Draw:** Players discard and draw again. Third betting round.
4. **Third Draw:** Final discard and draw. Fourth and final betting round.
5. **Showdown:** Lowest A-5 hand wins.

**Betting Structure:** Usually fixed-limit. Small bets on rounds 1–2, big bets on rounds 3–4.

### Razz (Stud Format)

**Street Structure (5 betting rounds):**

1. **Third Street:** Each player gets 2 cards face down, 1 card face up (the "door card"). The player with the highest door card posts the "bring-in" (forced bet). First betting round.
2. **Fourth Street:** One card dealt face up to each remaining player. Player with lowest exposed hand acts first. Second betting round.
3. **Fifth Street:** Another face-up card. Betting doubles to the big bet increment. Third betting round.
4. **Sixth Street:** Another face-up card. Fourth betting round.
5. **Seventh Street (River):** Final card dealt face down. Fifth and final betting round.
6. **Showdown:** Best five-card low hand from seven cards wins. (Players select 5 of their 7 cards.)

**Betting Structure:** Fixed-limit. Small bets on 3rd and 4th street, big bets on 5th, 6th, and 7th street. Maximum of 4 bets per street (bet, raise, re-raise, cap).

**Notes:**

- 2–8 players.
- No community cards — each player's cards are private/exposed individually.
- Suits only matter for determining who posts the bring-in (if two players tie for highest door card, the suit breaks the tie: spades > hearts > diamonds > clubs).
- In the evaluator, suits are irrelevant for hand ranking.

---

## No Limit Hold'em (NLHE)

**Goal:** Make the best possible five-card hand from 2 hole cards + 5 community cards (7 cards total, pick best 5). Standard high hand poker rankings.

**Ace Treatment:** Aces play both high and low for straights. A-K-Q-J-T is the highest straight (Broadway). A-2-3-4-5 is the lowest straight (the Wheel). However, K-A-2-3-4 is NOT a valid straight — the ace cannot wrap around.

**Straights:** Standard. Five consecutive ranks. Ace can bookend high or low but not wrap.

**Flushes:** Standard. Five cards of the same suit.

**Hand Construction:** Players may use any combination of their 2 hole cards and 5 community cards to make the best 5-card hand. This means they can use 0, 1, or 2 of their hole cards. ("Playing the board" uses 0 hole cards.)

**Hand Rankings (best to worst):**

1. Royal Flush (A-K-Q-J-T suited)
2. Straight Flush (five sequential same-suit cards)
3. Four of a Kind
4. Full House (three of a kind + pair)
5. Flush (five same-suit cards)
6. Straight (five sequential cards)
7. Three of a Kind
8. Two Pair
9. One Pair
10. High Card

**Street Structure (4 betting rounds):**

1. **Preflop:** Each player gets 2 hole cards face down. Blinds posted. Betting starts left of the big blind.
2. **Flop:** 3 community cards dealt face up. Second betting round, starting left of the dealer.
3. **Turn:** 1 community card dealt face up (4 total on board). Third betting round.
4. **River:** 1 community card dealt face up (5 total on board). Fourth and final betting round.
5. **Showdown:** Best 5-card hand from 7 available cards wins.

**Betting Structure:** No limit — any player may bet up to their full stack at any time. Minimum raise is the size of the previous raise or the big blind, whichever is larger.

**Suits:** Suits are never used to break ties in Hold'em. Identical hands split the pot.

---

## Pot Limit Omaha (PLO)

**Goal:** Make the best possible five-card hand using EXACTLY 2 hole cards + EXACTLY 3 community cards. Standard high hand poker rankings.

**Ace Treatment:** Same as Hold'em — aces play both high and low for straights.

**Straights:** Standard.

**Flushes:** Standard. But you MUST use exactly 2 hole cards. If there are 4 hearts on the board and you only have 1 heart in hand, you do NOT have a flush.

**Hand Construction — THE CRITICAL RULE:** Players MUST use exactly 2 of their 4 hole cards and exactly 3 of the 5 community cards. No more, no less. This is the single most important rule that differentiates Omaha from Hold'em, and the most commonly misunderstood rule by new players.

**Common Misreads:**

- Holding A-A-A-K on a board with another ace: you have three-of-a-kind (trips), NOT four-of-a-kind. You can only use 2 hole cards.
- Holding 3 spades in hand with 2 on the board: NOT a flush. You'd need exactly 2 spades from hand + 3 from board.
- Board shows a straight (e.g., 5-6-7-8-9): you cannot "play the board" — you must contribute exactly 2 cards from your hand.

**Hand Rankings:** Identical to Hold'em (Royal Flush through High Card).

**Street Structure:** Identical to Hold'em (Preflop → Flop → Turn → River). The only differences are:

- 4 hole cards dealt instead of 2.
- The "exactly 2 from hand" rule.
- Pot-limit betting structure.

**Betting Structure:** Pot limit — maximum bet is the current size of the pot. The pot size calculation: money already in the pot + (the amount needed to call × 3, if facing a bet). Minimum raise is the size of the previous bet or raise.

---

## Omaha Hi-Lo 8-or-Better (O8)

**Goal:** Split-pot game. The best high hand wins half the pot, and the best qualifying low hand wins the other half. If no low qualifies, the high hand scoops the entire pot.

**Hand Construction:** Same as PLO — EXACTLY 2 hole cards + EXACTLY 3 community cards. This rule applies independently to both the high and the low hand. You may use different 2-card combinations from your hand for high vs. low. You may also use the same cards for both.

### High Hand

Standard poker hand rankings, identical to PLO/Hold'em. Nothing special here.

### Low Hand

Uses **Ace-to-Five lowball** rankings with an **8-or-better qualifier:**

**Ace Treatment (for low):** Ace is always low. Ace is the best low card.

**Straights (for low):** Do NOT count against the low hand. A-2-3-4-5 is the best possible low AND a straight for the high hand simultaneously.

**Flushes (for low):** Do NOT count against the low hand.

**Qualifier:** A low hand must consist of 5 unpaired cards all ranked 8 or lower (8, 7, 6, 5, 4, 3, 2, A). If no player can make a qualifying low, the entire pot goes to the best high hand.

**Board Requirement:** There must be at least 3 cards ranked 8 or lower on the board with different ranks for a low to be possible (since you need 3 from the board).

**Best Low:** A-2-3-4-5 (the wheel). This hand is also a straight for the high side.

**Worst Qualifying Low:** 8-7-6-5-4.

**Low Ranking Logic:** Compare from highest card down. Lower is better. 7-5-4-3-2 beats 8-4-3-2-A because 7 < 8 as the high card. If highest cards match, compare second highest, and so on.

### Scooping and Quartering

A single player CAN win both high and low halves. This is called "scooping" and is the primary strategic goal in O8.

"Getting quartered" occurs when two players tie for the low (splitting the low half) while a third player wins the high. The quartered player wins only 25% of the pot despite having a qualifying hand.

### Counterfeiting

A key concept: if you hold A-2 and the board is 8-7-5, you have the nut low (A-2-5-7-8). But if a 2 falls on the turn, you're "counterfeited" — your hand is now A-2-5-7-8 (unchanged in absolute terms, but now someone holding A-3 has A-2-3-5-7 for a better low).

**Street Structure:** Identical to PLO/Hold'em (Preflop → Flop → Turn → River), with 4 hole cards.

**Betting Structure:** Most commonly fixed-limit. Also played pot-limit. Rarely no-limit.

---

## Badugi

**Goal:** Make the lowest possible four-card hand where all four cards have different suits AND different ranks. A complete 4-card hand with no duplicate suits or ranks is called a "Badugi."

**Ace Treatment:** Aces are always low. Ace is the best card.

**Straights:** Do NOT exist in Badugi. Having A-2-3-4 is perfect, not a straight. Sequential ranks are completely irrelevant.

**Flushes:** The concept doesn't apply in the traditional sense, but suits are structurally critical. Having two or more cards of the same suit means only the lowest one counts — the others are discarded from your hand evaluation.

**Hand Size / Card Elimination:** This is the core mechanic of Badugi. When evaluating a hand, any card that duplicates a suit or rank already present is removed (keeping the lowest card in case of conflict). The remaining cards form your "counting hand."

- **4-card Badugi:** All four cards are different suits and different ranks. Best possible hand type. Any 4-card Badugi beats any 3-card hand.
- **3-card hand:** Three cards count after removing one duplicate. Any 3-card hand beats any 2-card hand.
- **2-card hand:** Two cards count after removing two duplicates.
- **1-card hand:** Only one card counts (e.g., all four cards are the same suit).

**Within the same card count, compare from highest card down. Lower is better.**

**Best Hand:** A♣-2♦-3♥-4♠ (4-card Badugi, 4-high). Suits don't matter as long as they're all different.

**Second Best:** A♣-2♦-3♥-5♠ (4-card Badugi, 5-high).

**Worst 4-card Badugi:** A♣-J♦-Q♥-K♠ (King-high Badugi — still beats every 3-card hand).

**Example Hand Evaluations:**

- 2♠-4♦-6♥-9♣ → 4-card Badugi (9-high). All suits different, all ranks different.
- 5♠-4♦-8♠-3♥ → 3-card hand (8-4-3). Two spades — the 5♠ is discarded (keeping 3♥ since it's lower... actually, the rule is: of the two spades 5♠ and 8♠, the higher one 8♠ is discarded). Correction: of cards sharing a suit, only the lowest is kept. So 5♠ stays, 8♠ is removed. Hand becomes 3♥-4♦-5♠ → 3-card, 5-high.
- 3♦-3♠-7♥-K♣ → 3-card hand. The pair of 3s means one is removed. Keep 3♦ (or 3♠ — whichever leaves the better hand after also considering suits). Hand is 3-7-K three-card.

**Street Structure (4 betting rounds, 3 draws):**

1. **Pre-Draw:** Each player gets 4 cards face down. Blinds posted. First betting round.
2. **First Draw:** Players discard 0–4 cards and draw replacements. Second betting round.
3. **Second Draw:** Players discard and draw again. Third betting round.
4. **Third Draw:** Final discard and draw. Fourth and final betting round.
5. **Showdown:** Best Badugi hand wins.

**Betting Structure:** Typically fixed-limit. Small bets on rounds 1–2, big bets on rounds 3–4. Pot-limit Badugi also exists but is less common.

**Key Strategic Note:** Standing pat (drawing 0) signals a made Badugi. Drawing 1 signals a near-complete hand. Drawing 2+ signals a rebuild. Three-card hands often win at showdown — a complete 4-card Badugi is harder to make than it looks.

---

## Quick Comparison Table

| Game       | Cards Dealt          | Hand Size               | Ace                            | Straights                   | Flushes                     | Goal                                      | Streets                 |
| ---------- | -------------------- | ----------------------- | ------------------------------ | --------------------------- | --------------------------- | ----------------------------------------- | ----------------------- |
| NL 2-7     | 5                    | 5                       | Always high                    | Count against               | Count against               | Lowest hand                               | 2 (pre-draw, post-draw) |
| A-5 / Razz | 5 (draw) or 7 (stud) | 5                       | Always low                     | Ignored                     | Ignored                     | Lowest hand                               | 4 (draw) or 5 (stud)    |
| NLHE       | 2 + 5 board          | Best 5 of 7             | High and low                   | Standard                    | Standard                    | Highest hand                              | 4 (preflop–river)       |
| PLO        | 4 + 5 board          | Exactly 2+3             | High and low                   | Standard                    | Standard                    | Highest hand                              | 4 (preflop–river)       |
| O8         | 4 + 5 board          | Exactly 2+3             | High+low (hi), Always low (lo) | Standard (hi), Ignored (lo) | Standard (hi), Ignored (lo) | Split: best high + best low (8 qualifier) | 4 (preflop–river)       |
| Badugi     | 4                    | 1–4 (after elimination) | Always low                     | Don't exist                 | N/A (suits structural)      | Lowest Badugi                             | 4 (pre-draw + 3 draws)  |
