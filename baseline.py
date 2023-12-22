import json
import sys
from dataclasses import dataclass
from enum import Enum

MAX_INVEST_LEVEL = 20


@dataclass
class Project:
    h: int
    v: int


class CardType(Enum):
    WORK_SINGLE = 0
    WORK_ALL = 1
    CANCEL_SINGLE = 2
    CANCEL_ALL = 3
    INVEST = 4


@dataclass
class Card:
    t: CardType
    w: int
    p: int


class Judge:
    def __init__(self, n: int, m: int, k: int):
        self.n = n
        self.m = m
        self.k = k

    def read_initial_cards(self) -> list[Card]:
        cards = []
        for _ in range(self.n):
            t, w = map(int, input().split())
            cards.append(Card(CardType(t), w, 0))
        return cards

    def read_projects(self) -> list[Project]:
        projects = []
        for _ in range(self.m):
            h, v = map(int, input().split())
            projects.append(Project(h, v))
        return projects

    def use_card(self, c: int, m: int) -> None:
        print(f"{c} {m}", flush=True)

    def read_money(self) -> int:
        return int(input())

    def read_next_cards(self) -> list[Card]:
        cards = []
        for _ in range(self.k):
            t, w, p = map(int, input().split())
            cards.append(Card(CardType(t), w, p))
        return cards

    def select_card(self, r: int) -> None:
        print(r, flush=True)

    def comment(self, message: str) -> None:
        print(f"# {message}")


class Solver:
    def __init__(self, n: int, m: int, k: int, t: int):
        self.n = n
        self.m = m
        self.k = k
        self.t = t
        self.judge = Judge(n, m, k)

    def solve(self) -> int:
        self.turn = 0
        self.money = 0
        self.invest_level = 0
        self.cards = self.judge.read_initial_cards()
        self.projects = self.judge.read_projects()

        for _ in range(self.t):
            use_card_i, use_target = self._select_action()
            if self.cards[use_card_i].t == CardType.INVEST:
                self.invest_level += 1
            # example for comments
            self.judge.comment(f"used {self.cards[use_card_i]} to target {use_target}")
            self.judge.use_card(use_card_i, use_target)
            assert self.invest_level <= MAX_INVEST_LEVEL

            self.projects = self.judge.read_projects()
            self.money = self.judge.read_money()

            next_cards = self.judge.read_next_cards()
            select_card_i = self._select_next_card(next_cards)
            self.cards[use_card_i] = next_cards[select_card_i]
            self.judge.select_card(select_card_i)
            self.money -= next_cards[select_card_i].p
            assert self.money >= 0

            self.turn += 1

        return self.money

    def _select_action(self) -> tuple[int, int]:
        # TODO: implement your strategy
        return (0, 0)

    def _select_next_card(self, next_cards: list[Card]) -> int:
        # TODO: implement your strategy
        return 0


def main() -> None:
    n, m, k, t = map(int, input().split())
    solver = Solver(n, m, k, t)
    score = solver.solve()
    result = {"score": score, "duration": 0.1}
    print(f"result: {json.dumps(result)}", file=sys.stderr)


if __name__ == "__main__":
    main()
