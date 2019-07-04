#!/usr/bin/env python3

# usage: ./fetch.py | tee -a stats.csv
# post on /r/alitabattleangel about this script: https://www.reddit.com/r/alitabattleangel/comments/bj0okv/rankercom_statistics/
# spreadsheet containing all data: https://docs.google.com/spreadsheets/d/17uA2enNFaVfQevSEYOyF5Da0Ek8dJCRL7yM0sK54klY/edit#gid=1911187355

import time
from datetime import datetime
import sys
import csv
import requests

RANKER_LIST_ID = "298553"
RANKER_ITEM_ID = "85372114"
API_URL = "https://api.ranker.com/lists/{}/items/{}?include=crowdRankedStats,votes".format(
    RANKER_LIST_ID, RANKER_ITEM_ID
)
REQUEST_INTERVAL = 5 * 60  # seconds


def fetch_data():
    timestamp = datetime.utcnow()
    data = requests.get(API_URL).json()
    votes_data, reranks_data = data["votes"], data["crowdRankedStats"]
    return (
        timestamp,
        data["rank"],
        votes_data["upVotes"],
        votes_data["downVotes"],
        reranks_data["totalContributingListCount"],
        reranks_data["top5ListCount"],
    )


csv_writer = csv.writer(sys.stdout)

while True:
    t = time.time()

    row = fetch_data()
    timestamp, *data = row
    csv_writer.writerow([timestamp.strftime("%Y-%m-%d %H:%M:%S")] + data)
    sys.stdout.flush()

    while time.time() - t < REQUEST_INTERVAL:
        time.sleep(1)
