#!/usr/bin/env python3

# usage: GOOGLE_APPLICATION_CREDENTIALS=path/to/firebase-adminsdk-cridentials.json ./fetch.py | tee -a stats.csv
# post on /r/alitabattleangel about this script: https://www.reddit.com/r/alitabattleangel/comments/bj0okv/rankercom_statistics/
# spreadsheet containing all data: https://docs.google.com/spreadsheets/d/17uA2enNFaVfQevSEYOyF5Da0Ek8dJCRL7yM0sK54klY/edit#gid=1911187355

import time
from datetime import datetime
import sys
import csv
import requests
import firebase_admin
from firebase_admin import firestore
from compress import compress

RANKER_LIST_ID = "298553"
RANKER_ITEM_ID = "85372114"
API_URL = "https://cache-api.ranker.com/lists/{}/items/{}?include=crowdRankedStats,votes".format(
    RANKER_LIST_ID, RANKER_ITEM_ID
)
REQUEST_INTERVAL = 5 * 60  # seconds


firebase_admin.initialize_app()
db = firestore.client()
db_stats_collection = (
    db.collection("ranker").document(RANKER_LIST_ID).collection("stats")
)


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


def fetch_data_periodically():
    csv_writer = csv.writer(sys.stdout)

    while True:
        t = time.time()

        row = fetch_data()
        timestamp, *data = row
        csv_writer.writerow([timestamp.strftime("%Y-%m-%d %H:%M:%S")] + data)
        sys.stdout.flush()
        yield row

        while time.time() - t < REQUEST_INTERVAL:
            time.sleep(1)


for timestamp, rank, upvotes, downvotes, reranks, top5_reranks in compress(
    fetch_data_periodically(), lambda row: row[1:]
):
    db_stats_collection.add(
        {
            "timestamp": timestamp,
            "rank": rank,
            "upvotes": upvotes,
            "downvotes": downvotes,
            "reranks": reranks,
            "top5_reranks": top5_reranks,
        }
    )
