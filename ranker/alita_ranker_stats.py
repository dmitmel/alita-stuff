#!/usr/bin/env python3

# usage: python3 alita_ranker_stats.py | tee -a alita_ranker_stats.csv
# post on /r/alitabattleangel about this script: https://www.reddit.com/r/alitabattleangel/comments/bj0okv/rankercom_statistics/
# spreadsheet containing all data: https://docs.google.com/spreadsheets/d/17uA2enNFaVfQevSEYOyF5Da0Ek8dJCRL7yM0sK54klY/edit#gid=1911187355

import time
from datetime import datetime
import sys
import csv
import requests

API_URL = "https://cache-api.ranker.com/lists/298553/items/85372114?include=crowdRankedStats,votes"


writer = csv.writer(sys.stdout)

while True:
    t = time.time()

    data = requests.get(API_URL).json()
    votes_data, reranks_data = data["votes"], data["crowdRankedStats"]
    writer.writerow(
        [
            datetime.utcfromtimestamp(t).strftime("%Y-%m-%d %H:%M:%S"),
            data["rank"],
            votes_data["upVotes"],
            votes_data["downVotes"],
            reranks_data["totalContributingListCount"],
            reranks_data["top5ListCount"],
        ]
    )
    sys.stdout.flush()

    while time.time() - t < 60:
        time.sleep(0.1)
