#!/usr/bin/env python3
"""One-off generator for cards/core-set/*.ron from the modeled card data below."""

import re
from pathlib import Path

OUT_DIR = Path("cards/core-set")


def load_card_named_variants():
    """Map a human card name (e.g. "Port of Call") to its CardNamed variant
    (e.g. "PortofCall") by matching normalized (lowercase, alnum-only) forms
    against the enum declared in src/main.rs."""
    main_rs = Path("src/main.rs").read_text()
    m = re.search(r"enum CardNamed \{(.*?)\}", main_rs, re.S)
    variants = [v.strip() for v in m.group(1).split(",") if v.strip()]

    def norm(s):
        return re.sub(r"[^a-z0-9]", "", s.lower())

    return {norm(v): v for v in variants}


CARD_NAMED_BY_NORM = load_card_named_variants()


def card_named(name):
    key = re.sub(r"[^a-z0-9]", "", name.lower())
    return CARD_NAMED_BY_NORM[key]


class V:
    """A bare enum variant, or a tuple-variant call: Name(arg1, arg2, ...)."""

    def __init__(self, name, *args):
        self.name = name
        self.args = args


class F:
    """A struct-variant call: Name(field: value, field2: value2)."""

    def __init__(self, name, **fields):
        self.name = name
        self.fields = fields


class L:
    """A RON list literal."""

    def __init__(self, *items):
        self.items = list(items)


def render(node, indent):
    pad = "    " * indent
    inner_pad = "    " * (indent + 1)

    if node is None:
        return "None"
    if isinstance(node, bool):
        return "true" if node else "false"
    if isinstance(node, int):
        return str(node)
    if isinstance(node, str):
        return node  # bare identifier (enum variant with no data)

    if isinstance(node, V):
        if not node.args:
            return node.name
        if node.name == "Sequence":
            (lst,) = node.args
            items = "".join(f"{inner_pad}{render(i, indent + 1)},\n" for i in lst.items)
            return f"Sequence([\n{items}{pad}])"
        rendered = ", ".join(render(a, indent) for a in node.args)
        return f"{node.name}({rendered})"

    if isinstance(node, F):
        rendered = ", ".join(f"{k}: {render(v, indent)}" for k, v in node.fields.items())
        return f"{node.name}({rendered})"

    if isinstance(node, L):
        rendered = ", ".join(render(i, indent) for i in node.items)
        return f"[{rendered}]"

    raise TypeError(f"unhandled node type: {node!r}")


def card(name, faction, card_type, cost, effect, is_all_faction_ally=False, note=None):
    lines = ["("]
    lines.append(f"    name: {card_named(name)},")
    lines.append(f"    faction: {faction},")
    lines.append(f"    card_type: {render(card_type, 1)},")
    lines.append(f"    cost: {cost},")
    if note:
        for n in note if isinstance(note, list) else [note]:
            lines.append(f"    // {n}")
    lines.append(f"    effect: {render(effect, 1)},")
    if is_all_faction_ally:
        lines.append("    is_all_faction_ally: true,")
    lines.append(")\n")
    return "\n".join(lines)


def Ship():
    return V("Ship")


def Base(defense, outpost):
    return F("Base", defense=defense, it_outpost=outpost)


def Res(rtype, n):
    return V("Base", V("Resource", rtype, n))


def TE(inner):
    return V("Base", inner)


def If(cond, eff):
    return V("If", cond, eff)


def Ally(faction, eff):
    return If(V("HasAlly", faction), eff)


def May(eff):
    return V("May", eff)


def Or_(a, b):
    return V("Or", a, b)


def AndOr(a, b):
    return V("AndOr", a, b)


def Seq(*items):
    return V("Sequence", L(*items))


def On(event, eff):
    return V("On", event, eff)


def Draw(n):
    return V("Draw", V("Number", n) if isinstance(n, int) else n)


def ShipsPlayed(faction):
    return V("ShipsPlayed", faction)


def Scrap(scrap_type, n):
    return TE(V("Scrap", scrap_type, n))


def Discard(n):
    return TE(V("Discard", n))


ON_SCRAP = "Scrap"

CARDS = []


def add(*args, **kwargs):
    CARDS.append(card(*args, **kwargs))


# ---------------------------------------------------------------- Blob ----

add("Battle Blob", "Blob", Ship(), 6, Seq(
    Res("Combat", 8),
    Ally("Blob", Draw(1)),
    On(ON_SCRAP, Res("Combat", 4)),
))

add("Battle Pod", "Blob", Ship(), 2, Seq(
    Res("Combat", 4),
    May(TE(V("ScrapCardInRow"))),
    Ally("Blob", Res("Combat", 2)),
))

add("Blob Carrier", "Blob", Ship(), 6, Seq(
    Res("Combat", 7),
    Ally("Blob", TE(F("AquireShipForFree", to_top_of_deck=True, max_cost=None))),
))

add("Blob Destroyer", "Blob", Ship(), 4, Seq(
    Res("Combat", 6),
    Ally("Blob", May(AndOr(TE(V("DestroyTargetBase")), TE(V("ScrapCardInRow"))))),
))

add("Blob Fighter", "Blob", Ship(), 1, Seq(
    Res("Combat", 3),
    Ally("Blob", Draw(1)),
))

add("Blob Wheel", "Blob", Base(5, False), 3, Seq(
    Res("Combat", 1),
    On(ON_SCRAP, Res("Trade", 3)),
))

add("Blob World", "Blob", Base(7, False), 8, Or_(Res("Combat", 5), Draw(ShipsPlayed("Blob"))))

add("Mothership", "Blob", Ship(), 7, Seq(
    Res("Combat", 6),
    Draw(1),
    Ally("Blob", Draw(1)),
))

add("Ram", "Blob", Ship(), 3, Seq(
    Res("Combat", 5),
    Ally("Blob", Res("Combat", 2)),
    On(ON_SCRAP, Res("Trade", 3)),
))

add("The Hive", "Blob", Base(5, False), 5, Seq(
    Res("Combat", 3),
    Ally("Blob", Draw(1)),
))

add("Trade Pod", "Blob", Ship(), 2, Seq(
    Res("Trade", 3),
    Ally("Blob", Res("Combat", 2)),
))

# ---------------------------------------------------------- Machine Cult --

add("Battle Mech", "MachineCult", Ship(), 5, Seq(
    Res("Combat", 4),
    May(Scrap("HandOrDiscardPile", 1)),
    Ally("MachineCult", Draw(1)),
))

add("Battle Station", "MachineCult", Base(5, True), 3,
    On(ON_SCRAP, Res("Combat", 5)))

add("Brain World", "MachineCult", Base(6, True), 8,
    May(Or_(
        Seq(Scrap("HandOrDiscardPile", 1), Draw(1)),
        Seq(Scrap("HandOrDiscardPile", 2), Draw(2)),
    )))

add("Junkyard", "MachineCult", Base(5, True), 6, May(Scrap("HandOrDiscardPile", 1)))

add("Machine Base", "MachineCult", Base(6, True), 7, Seq(
    Draw(1),
    Scrap("Hand", 1),
))

add("Mech World", "MachineCult", Base(6, True), 5, Seq(), is_all_faction_ally=True)

add("Missile Bot", "MachineCult", Ship(), 2, Seq(
    Res("Combat", 2),
    May(Scrap("HandOrDiscardPile", 1)),
    Ally("MachineCult", Res("Combat", 2)),
))

add("Missile Mech", "MachineCult", Ship(), 6, Seq(
    Res("Combat", 6),
    May(TE(V("DestroyTargetBase"))),
    Ally("MachineCult", Draw(1)),
))

add("Patrol Mech", "MachineCult", Ship(), 4, Seq(
    Or_(Res("Trade", 3), Res("Combat", 5)),
    Ally("MachineCult", May(Scrap("HandOrDiscardPile", 1))),
))

add("Stealth Needle", "MachineCult", Ship(), 4, V("CopyPlayedShip"))

add("Supply Bot", "MachineCult", Ship(), 3, Seq(
    Res("Trade", 2),
    May(Scrap("HandOrDiscardPile", 1)),
    Ally("MachineCult", Res("Combat", 2)),
))

add("Trade Bot", "MachineCult", Ship(), 1, Seq(
    Res("Trade", 1),
    May(Scrap("HandOrDiscardPile", 1)),
    Ally("MachineCult", Res("Combat", 2)),
))

# ------------------------------------------------------------ Star Empire -

add("Battlecruiser", "StarEmpire", Ship(), 6, Seq(
    Res("Combat", 5),
    Draw(1),
    Ally("StarEmpire", TE(V("OpponentDiscards"))),
    On(ON_SCRAP, Seq(Draw(1), May(TE(V("DestroyTargetBase"))))),
))

add("Corvette", "StarEmpire", Ship(), 2, Seq(
    Res("Combat", 1),
    Draw(1),
    Ally("StarEmpire", Res("Combat", 2)),
))

add("Dreadnaught", "StarEmpire", Ship(), 7, Seq(
    Res("Combat", 7),
    Draw(1),
    On(ON_SCRAP, Res("Combat", 5)),
))

add("Fleet HQ", "StarEmpire", Base(8, False), 8,
    On(V("PlayShip", "Unaligned"), Res("Combat", 1)))

add("Imperial Fighter", "StarEmpire", Ship(), 1, Seq(
    Res("Combat", 2),
    TE(V("OpponentDiscards")),
    Ally("StarEmpire", Res("Combat", 2)),
))

add("Imperial Frigate", "StarEmpire", Ship(), 3, Seq(
    Res("Combat", 4),
    TE(V("OpponentDiscards")),
    Ally("StarEmpire", Res("Combat", 2)),
    On(ON_SCRAP, Draw(1)),
))

add("Recycling Station", "StarEmpire", Base(4, True), 4, Or_(
    Res("Trade", 1),
    May(Or_(
        Seq(Discard(1), Draw(1)),
        Seq(Discard(2), Draw(2)),
    )),
))

add("Royal Redoubt", "StarEmpire", Base(6, True), 6, Seq(
    Res("Combat", 3),
    Ally("StarEmpire", TE(V("OpponentDiscards"))),
))

add("Space Station", "StarEmpire", Base(4, True), 4, Seq(
    Res("Combat", 2),
    Ally("StarEmpire", Res("Combat", 2)),
    On(ON_SCRAP, Res("Trade", 4)),
))

add("Survey Ship", "StarEmpire", Ship(), 3, Seq(
    Res("Trade", 1),
    Draw(1),
    On(ON_SCRAP, TE(V("OpponentDiscards"))),
))

add("War World", "StarEmpire", Base(4, True), 5, Seq(
    Res("Combat", 3),
    Ally("StarEmpire", Res("Combat", 4)),
))

# --------------------------------------------------------- Trade Federation

add("Barter World", "TradeFederation", Base(4, False), 4, Seq(
    Or_(Res("Authority", 2), Res("Trade", 2)),
    On(ON_SCRAP, Res("Combat", 5)),
))

add("Central Office", "TradeFederation", Base(6, False), 7, Seq(
    Res("Trade", 2),
    May(V("NextAcquiredShipToTopOfDeck")),
    Ally("TradeFederation", Draw(1)),
))

add("Command Ship", "TradeFederation", Ship(), 8, Seq(
    Res("Authority", 4),
    Res("Combat", 5),
    Draw(2),
    Ally("TradeFederation", TE(V("DestroyTargetBase"))),
))

add("Cutter", "TradeFederation", Ship(), 2, Seq(
    Res("Authority", 4),
    Res("Trade", 2),
    Ally("TradeFederation", Res("Combat", 4)),
))

add("Defense Center", "TradeFederation", Base(5, True), 5, Seq(
    Or_(Res("Authority", 3), Res("Combat", 2)),
    Ally("TradeFederation", Res("Combat", 2)),
))

add("Embassy Yacht", "TradeFederation", Ship(), 3, Seq(
    Res("Authority", 3),
    Res("Trade", 2),
    If(V("BaseCountAtLeast", 2), Draw(2)),
))

add("Federation Shuttle", "TradeFederation", Ship(), 1, Seq(
    Res("Trade", 2),
    Ally("TradeFederation", Res("Authority", 4)),
))

add("Flagship", "TradeFederation", Ship(), 6, Seq(
    Res("Combat", 5),
    Draw(1),
    Ally("TradeFederation", Res("Authority", 5)),
))

add("Freighter", "TradeFederation", Ship(), 4, Seq(
    Res("Trade", 4),
    Ally("TradeFederation", May(V("NextAcquiredShipToTopOfDeck"))),
))

add("Port of Call", "TradeFederation", Base(6, True), 6, Seq(
    Res("Trade", 3),
    On(ON_SCRAP, Seq(Draw(1), May(TE(V("DestroyTargetBase"))))),
))

add("Trade Escort", "TradeFederation", Ship(), 5, Seq(
    Res("Authority", 4),
    Res("Combat", 4),
    Ally("TradeFederation", Draw(1)),
))

add("Trading Post", "TradeFederation", Base(4, True), 3, Seq(
    Or_(Res("Authority", 1), Res("Trade", 1)),
    On(ON_SCRAP, Res("Combat", 3)),
))

# --------------------------------------------------------------- Unaligned

add("Explorer", "Unaligned", Ship(), 2, Seq(
    Res("Trade", 2),
    On(ON_SCRAP, Res("Combat", 2)),
))

add("Scout", "Unaligned", Ship(), 0, Res("Trade", 1))

add("Viper", "Unaligned", Ship(), 0, Res("Combat", 1))


def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    names = set()
    for text in CARDS:
        name = text.splitlines()[1].split('"')[1]
        names.add(name)
        (OUT_DIR / f"{name}.ron").write_text(text, encoding="utf-8")
    print(f"wrote {len(names)} cards")


if __name__ == "__main__":
    main()
