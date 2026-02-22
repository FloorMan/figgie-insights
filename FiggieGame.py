import random
import matplotlib.pyplot as plt


class gameSpace:
    def __init__(self, players):
        self.players = []
        self.commonSuit = "Diamonds"
        self.goalSuit = "Hearts"
        self.cardDistributions = {"Spades" : 8, "Clubs" : 10, "Hearts" : 10, "Diamonds" : 12}
        self.players = players
        self.cards_per_player = 10

    def dealCards(self):
        for player_instance in self.players:
            player_instance.cards = {
                "Spades": 0,
                "Clubs": 0,
                "Hearts": 0,
                "Diamonds": 0,
            }

        deck = []
        for suit, count in self.cardDistributions.items():
            deck.extend([suit] * count)

        total_needed = self.cards_per_player * len(self.players)
        if total_needed > len(deck):
            raise ValueError("Not enough cards to deal 10 to each player.")

        random.shuffle(deck)

        for i in range(total_needed):
            suit = deck[i]
            self.players[i % len(self.players)].cards[suit] += 1

    def changeCommonSuit(self):
        suits = ['Spades', 'Clubs', 'Hearts', 'Diamonds']
        goalSuit = {"Spades" : "Clubs", "Clubs" : "Spades", "Hearts" : "Diamonds", "Diamonds" : "Hearts"}
        numCards = [8, 10, 10]
        self.commonSuit = random.choice(suits)
        self.goalSuit = goalSuit[self.commonSuit]


        self.cardDistributions[self.commonSuit] = 12
        suits.remove(self.commonSuit)

        for suit in suits:
            cards = random.choice(numCards)
            self.cardDistributions[suit] = cards
            numCards.remove(cards)


    def displayCards(self):
        print(f"Card distributions:")
        print(self.cardDistributions)
        print("Common Suit: ", self.commonSuit)
        print("Goal Suit: ", self.goalSuit)
        
        print("Player cards: ")
        for i in range(len(self.players)):
            print(f"player {i}: ", self.players[i].cards)

        print("\n-----------------------------------------------\n")

    def newGame(self):
        self.changeCommonSuit()
        self.dealCards()

    def calculateProbabilities(self):
        if not hasattr(self, "total_goal_guesses"):
            self.total_goal_guesses = 0
            self.correct_goal_guesses = 0
            self.games_played = 0

        goalSuitMap = {
            "Spades": "Clubs",
            "Clubs": "Spades",
            "Hearts": "Diamonds",
            "Diamonds": "Hearts",
        }

        for player_instance in self.players:
            max_count = max(player_instance.cards.values())
            top_suits = [
                suit for suit, count in player_instance.cards.items()
                if count == max_count
            ]

            if len(top_suits) != 1:
                continue

            assumed_common_suit = top_suits[0]
            guessed_goal_suit = goalSuitMap[assumed_common_suit]

            self.total_goal_guesses += 1
            if guessed_goal_suit == self.goalSuit:
                self.correct_goal_guesses += 1

        self.games_played += 1

        if self.total_goal_guesses == 0:
            return 0

        return self.correct_goal_guesses / self.total_goal_guesses

    def visualizeProbabilities(self, games_to_play):
        probabilities = []
        self.newGame()
        for _ in range(games_to_play):
            for i in range(5):
                self.calculateProbabilities()
                self.newGame()
            probabilities.append(self.calculateProbabilities())

        plt.figure(figsize=(10, 5))
        plt.plot(range(1, games_to_play + 1), probabilities, label="Running probability")
        plt.xlabel("Games played")
        plt.ylabel("Probability of correct goal-suit guess")
        plt.title("Goal-suit guess probability over games")
        plt.grid(True, alpha=0.3)
        plt.legend()
        plt.tight_layout()
        plt.show()

        return probabilities
            


class player:
    def __init__(self):
        self.cards = {"Spades" : 0, "Clubs" : 0, "Hearts" : 0, "Diamonds" : 0}


players = []
for i in range(4):
    players.append(player())


game = gameSpace(players)
game.visualizeProbabilities(200)




