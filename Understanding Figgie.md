[[Quant Training]]
[[Traders in a Strange Land]]

Trying to understand the math behind Figgie, and optimizing to win. 

Rules: [figgie.com](https://www.figgie.com/how-to-play.html)
The website I'm reading through: [Understanding Figgie](https://www.loganseaburg.com/blog/figgie-1-background#user-content-fnref-2)
Good series on someone’s first intuition on the game: [Figgie Day 2 - Strategic Formalization](https://www.youtube.com/watch?v=2GiLRKDe_w4)

---
# ✅To Do
- [ ] Finish statistical analysis on game
	- [x] Probabilities
	- [x] Expected Values
	- [x] Marginal value of trades
- [ ] Create first couple of strategies
- [ ] Finish rough simulation
- [ ] Create dashboard w/ results
- [ ] Create auto-strategy bots
- [ ] Create backend for server-client model
	- [ ] Put into Docker container
- [ ] Create database for results
- [ ] GUI for live-game visualization
- [ ] WebSocket Server Support
- [ ] Reinforcement Learning Model
	- [ ] Varius ML based models
## Nice to have
- [x] Read through ***Traders in a Strange Land*** and implement bots
---
# Statistical Analysis
To understand how to get better at the game, we must first understand our odds. Seeing how the cards are distributed can give us a leg up when making decisions. 
## Bayesian Inferences

### Starting Hand
The most important suit in the game is the target suit. This is the only suit that get’s scored at the end of the round, and can often swing a losing round into a winning game. To find the target suit, we need to find the common suit, which will always has 12 cards.

The common suit will be a randomly picked suit, being made up of 12 cards. Since the common suit has the most amount of cards, implicitly we know that there is a greater likelihood of the common suit than any other suit. 

In Figgie, the common suit is hidden from us. If we draw 6 hearts, 2 spades, 1 diamond, and 1 club, we would assume that hearts is the common suit, since we are more likely to be dealt a higher number of the common suit than any other suit. But what are the odds exactly? What is the probability of hearts being the common suit given I’m dealt $n$ hearts?

Using Bayes’ formula, we can calculate the conditional odds of a single suit being common, given a certain amount of cards being dealt. 
$$P(A|B) = \frac{P(B|A) \cdot P(A)}{P(B)}$$
For a given number of cards dealt $K$ from suit $S$, we want to determine the probability that $S$ is the common suit $C$:
$$P(C = S | K_S = n) = \frac{P(K_S = n|C = S) \cdot P(C=S)}{P(K_S = n)}$$

The probability that our given suit is the common suit is equally distributed:
$$P(C=S) = \frac{1}{\text{Total Suits}} = \frac{1}{4}$$

The probability of $n$ cards being dealt for suit $S$ is given by the number of permutations of $n$ cards over the total ways to deal cards for the suit $S$. This gets particularly tricky when we consider that our suit could take on 12, 10, or 8 cards in our total hand. To find our joint probability, we can add together the case where our suit $S$ is common, and the cases where $S$ is not common.

In the case where $S$ is common, there is only one way that the card layout could exist ($S$ is made up of 12 cards total). We find the number of ways to order $n$ cards from $S$, given the common suit is $S$.
$$\text{Combination of n cards from S | Common} = {12 \choose n} \cdot {40-12 \choose 10-n}$$
To find the probability, we divide these combination by the total amount of hands:
$$P(K_S=n | C=S) = \frac{\text{Combination of n cards from S | Common}}{\text{Total Hands}} = \frac{{12 \choose n} \cdot {40-12 \choose 10-n}}{{40 \choose 10}}$$


In the case where $S$ is not common, there are 2 different ways that $S$ could be made up: 10 cards and 8 cards. 
$$\text{Combination of n cards given S is not common, made of 10} = {10 \choose n} \cdot {40-10 \choose 10-n}$$
$$\text{Combination of n cards given S is not common, made of 8} = {8 \choose n} \cdot {40-8 \choose 10-n}$$
We add together these two combinations to form our total event space, and divide by our total hands possible:
$$P(K_S=n | C \neq S) = \frac{\text{Combination of n cards from S | Not Common, 10 } + \text{Combination of n cards from S | Not Common, 8 }}{\text{Total Hands}}$$
$$P(K_S = n | C \neq S) = \frac{{10 \choose n} \cdot {40-10 \choose 10-n} + {8 \choose n} \cdot {40-8 \choose 10-n}}{{40 \choose 10}}$$

To find the joint probability of $n$ being dealt, we add together these probabilities by their weights:
$$P(K_S = n) = P(K_S=n|C=S) \cdot P(C=S) + P(K_S=n|C \neq S) \cdot P(C \neq S)$$
Finally, we can reconstruct our probability of $S$ being the common suit:
$$P(C = S | K_S = n) = \frac{P(K_S = n|C = S) \cdot P(C=S)}{P(K_S = n)}$$
Below is the result of our calculations for each number $n$ dealt.
![[SpadesBeingCommonGivenNSpades.png|800]]
### All Cards Give us Information
We also have to consider the other cards in our starting hand, as this could also give us a leg up when evaluating our probability. 
What happens if we have 7 Spades, and 2 Hearts. How does this change our probability? 

![[2SuitsProbability.png|800]]

![[2SuitsProbabilityHeatmap.png|800]]

---
## Expected Value and Rational Pricing
One cannot win on probability alone. We have currently calculated the approximate probability of having the common suit on first draw, but the aim of the game is to end up with the most amount of money. Knowing the common suit is only helpful because of the ability to find the goal suit, which is the only suit that awards chips at the end of the round.

Going off of our priors (we will implement a model for updating our priors later on), we will find the marginal value of buying chips, the expected payout at the end of a round, and determine a policy to find the best price for buying chips.

### Marginal Value per Chip
The marginal value of a card is the expected value of the end-of-round payoff that buying a single chip will give us.  Let X be the goal suit:
$$\mathbb{E}[X] = \sum_{X=x}{(r_x \cdot P(x))} = r_{T=X} \cdot P(T = X) + r_{T \neq X} \cdot P(T \neq X)$$
Since there is only a reward for the target suit (+10 chips), and all other suits receive nothing (0 chips), we can exclude the second term of our summation.
$$\mathbb{E}[X] = r_{T=X} \cdot P(T=X)$$
Finally, we know that the $P(\text{Target})$ is given by the probability of the alternate suit ($X_\text{alt}$ )being the common suit:
$$\mathbb{E}[X] = r_{T=X} \cdot P(C=X_\text{alt.})$$
Knowing the expected value of each chip, we can come up with a simple policy:
- If $\mathbb{E}[X] > \text{Buy Price}_X \rightarrow \text{Buy}$
- If $\mathbb{E}[X] < \text{Buy Price}_X \rightarrow \text{Don't Buy}$
### Marginal Value per Chip on Total Pot
At the beginning of the round, each player contributes $50 to the pot (Or however much they have left if under $50). At the end of the round, the player who holds the majority of the target suit will win the pot. Thus, we need to calculate the marginal value of a card at this majority threshold to properly understand our odds.






---
## Efficiency + Information Flow





## To Do
- [x] Probability of Common suit given $n$ suit
	- [x] Probability of common suit given whole starting hand
- [ ] Sequential Bayesian Updating
	- [ ] How does market info change posterior belief
- [ ] Value of Information
	- [ ] Quantify value of knowing your hand
	- [ ] information gain and monetary value of entropy reduction
- [ ] Optimal distribution of hand at EOR
	- [ ] Payout for guessing correctly
	- [ ] Expected value of each card
	- [ ] Payout weighted with probability
- [ ] Does price converge to true value?
- [ ] Probability of every hand

### Visualizations
- [ ] Heatmap: Posterior probability vs hand composition
- [ ] 3d Surface: Expected value of suit vs posterior
- [ ] Convergence curve: price → true value
- [ ] Simulation distribution of profits by strategy
- [ ] Entropy decay over time

---
# 📕Additional Reading Material

- [x] [Understanding Jane Street](https://web.archive.org/web/20221230205817/https://www.thediff.co/archive/jane-street/)
- [x] Jane Street Figgie Champion and his thoughts | [Poker is a bad game for teaching epistemics. Figgie is a better one.](https://blog.rossry.net/figgie/)
- [x] Deprecated GitHub Figgie bot | [figgiebot](https://github.com/CodingYuno/figgiebot/tree/main)
- [ ] Paper outlining agent strategies for baseline performance | [Traders in a Strange Land](https://arxiv.org/pdf/2110.00879)
- [ ] GitHub repo for creating a FiggieBot backend | [FiggieBot](https://github.com/CornellDataScience/FiggieBot/tree/main)
- [ ] GitHub repo for Figgie trading strategies | [figgie-auto](https://github.com/0xDub/figgie-auto)
- [x] Jane Street Podcast | [Signals & Threads](https://signalsandthreads.com/)
- [ ] GitHub Figgie Server from blog writeup | [FiggieServer](https://github.com/LSeaburg/FiggieServer)
- [ ] Hypergeometric and Negative Binomial Distributions | [PDF Notes](https://dylanspicker.com/courses/STAT2593/content/Lesson%20014%20-%20Slides.pdf)
	- [ ] STAT2593 Probability and Statistics for Engineers | [Dylan Spicker Website](https://dylanspicker.com/courses/STAT2593/course_index.html)
	- [ ] 
