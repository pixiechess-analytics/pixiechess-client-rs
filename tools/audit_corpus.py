"""
Audit the vendored fixture corpus (tests/fixtures/pixiechess-api.json) to drive
typed-model tightening. Prints a per-field verdict for every endpoint:

  REQUIRED — present in 100% of examples, never null → tighten to T
  OPTIONAL — sometimes missing or null → keep Option<T>

Run from the repo root:
    python3 tools/audit_corpus.py
"""

import json
import re
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
FIX = REPO / "tests" / "fixtures" / "pixiechess-api.json"

# (endpoint, body-path, model-label). body-path follows the captured response.body:
#   ""              — top-level body
#   ".auction"      — body["auction"]
#   ".auctions[]"   — every element of body["auctions"]
#   "[]"            — body is an array; every element
#   ".days[]"       — every element of body["days"]
TARGETS = [
    ("GET /auction/{address}",                          ".auction",                  "Auction"),
    ("GET /auctions/active",                            ".auctions[]",               "Auction"),
    ("GET /auctions/daily-volume",                      ".days[]",                   "DailyVolume"),
    ("GET /auctions/last-completed-day-summary",        "",                          "CompletedDaySummary"),
    ("GET /auctions/past",                              "",                          "PastAuctionsPage"),
    ("GET /auctions/past",                              ".dayBuckets[].auctions[]",  "PastAuctionEntry"),
    ("GET /auctions/piece/{pieceKey}",                  "",                          "AuctionPieceInfo"),
    ("GET /auctions/piece/{pieceKey}/daily-volume",     ".days[]",                   "DailyVolume"),
    ("GET /auctions/today-summary",                     "",                          "AuctionDaySummary"),
    ("GET /burned-pieces/{address}",                    "",                          "PiecesPage (burned)"),
    ("GET /burned-pieces/{address}",                    ".pieces[]",                 "Piece (burned)"),
    ("GET /config/public",                              "",                          "PublicConfig"),
    ("GET /eth-usd-price",                              "",                          "EthUsdPrice"),
    ("GET /game/{gameId}",                              "",                          "Game"),
    ("GET /game/{gameId}/rating/{address}",             "",                          "RatingChange"),
    ("GET /leaderboard",                                "",                          "LeaderboardPage"),
    ("GET /leaderboard",                                ".entries[]",                "LeaderboardEntry"),
    ("GET /leaderboard",                                ".currentUser",              "LeaderboardEntry (currentUser variant)"),
    ("GET /live-feed",                                  "[]",                        "LiveFeedEvent"),
    ("GET /pieces/{address}",                           "",                          "PiecesPage"),
    ("GET /pieces/{address}",                           ".pieces[]",                 "Piece"),
    ("GET /points-leaderboard",                         "",                          "PointsLeaderboardPage"),
    ("GET /points-leaderboard",                         ".entries[]",                "PointsLeaderboardEntry"),
    ("GET /points-leaderboard",                         ".currentUser",              "PointsLeaderboardEntry (currentUser)"),
    ("GET /prices",                                     "",                          "Prices"),
    ("GET /ranks/masters",                              "",                          "{addresses:[]}"),
    ("GET /tournament/details/{tournamentId}",          ".data",                     "Tournament (details)"),
    ("GET /tournament/list",                            ".tournaments[]",            "Tournament (list)"),
    ("GET /tournament/waitlist/{tournamentId}",         "[]",                        "WaitlistEntry"),
    ("GET /user/match-history/{address}",               "",                          "MatchHistoryPage"),
    ("GET /user/match-history/{address}",               ".matches[]",                "MatchHistoryEntry"),
    ("GET /user/{userId}",                              ".user",                     "User"),
    ("GET /vault-balance",                              "",                          "{balance:String}"),
]


def value_kind(v):
    if v is None: return "null"
    if isinstance(v, bool): return "bool"
    if isinstance(v, int): return "int"
    if isinstance(v, float): return "float"
    if isinstance(v, str): return "string"
    if isinstance(v, list): return "array"
    if isinstance(v, dict): return "object"
    return type(v).__name__


TOKEN_RE = re.compile(r"\.([A-Za-z_][A-Za-z0-9_]*)|\[\]")


def tokens_of(path: str):
    out = []
    for m in TOKEN_RE.finditer(path):
        if m.group(1) is not None:
            out.append(("field", m.group(1)))
        else:
            out.append(("arr", None))
    return out


def walk(tokens, node):
    if not tokens:
        if node is not None:
            yield node
        return
    head, rest = tokens[0], tokens[1:]
    if head[0] == "field":
        if isinstance(node, dict):
            sub = node.get(head[1])
            if sub is not None:
                yield from walk(rest, sub)
    else:
        if isinstance(node, list):
            for it in node:
                yield from walk(rest, it)


def report(records, label):
    if not records:
        print(f"\n=== {label}: no records ===")
        return
    n = len(records)
    fields = defaultdict(
        lambda: {"present": 0, "null": 0, "types": set(), "arr_elem_types": set()}
    )
    for rec in records:
        if not isinstance(rec, dict):
            continue
        for k, v in rec.items():
            f = fields[k]
            f["present"] += 1
            kind = value_kind(v)
            f["types"].add(kind)
            if kind == "null":
                f["null"] += 1
            if isinstance(v, list):
                for elem in v:
                    f["arr_elem_types"].add(value_kind(elem))
    print(f"\n=== {label} ({n} records) ===")
    rows = []
    for k, f in fields.items():
        pres = 100 * f["present"] / n
        null = 100 * f["null"] / n if n else 0
        types = sorted(t for t in f["types"] if t != "null")
        verdict = "REQUIRED" if pres == 100 and f["null"] == 0 else "OPTIONAL"
        extra = ""
        if f["arr_elem_types"]:
            ats = sorted(f["arr_elem_types"])
            extra = f" [arr: {','.join(ats)}]"
        rows.append((verdict, k, f"{pres:.0f}%", f"{null:.0f}%", ",".join(types) + extra))
    rows.sort(key=lambda r: (r[0] != "REQUIRED", r[1]))
    for verdict, k, pres, null, types in rows:
        print(f"  {verdict:8s}  {k:38s}  pres={pres:>4s}  null={null:>4s}  types={types}")


def main():
    corpus = json.load(FIX.open())
    for endpoint, path, label in TARGETS:
        group = corpus.get(endpoint)
        if not group:
            continue
        bodies = [ex["response"]["body"] for ex in group["examples"]]
        tokens = tokens_of(path)
        records = []
        for body in bodies:
            records.extend(list(walk(tokens, body)))
        report(records, f"{endpoint} {path or '(top)'} → {label}")


if __name__ == "__main__":
    main()
