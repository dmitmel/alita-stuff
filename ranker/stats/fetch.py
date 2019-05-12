#!/usr/bin/env python3

# post on /r/alitabattleangel about this script: https://www.reddit.com/r/alitabattleangel/comments/bj0okv/rankercom_statistics/
# spreadsheet containing all data: https://docs.google.com/spreadsheets/d/17uA2enNFaVfQevSEYOyF5Da0Ek8dJCRL7yM0sK54klY/edit#gid=1911187355

import time
from datetime import datetime
import sys
import csv
import requests
import firebase_admin
from firebase_admin import firestore

RANKER_LIST_ID = "298553"
RANKER_ITEM_ID = "85372114"
API_URL = "https://cache-api.ranker.com/lists/{}/items/{}?include=crowdRankedStats,votes".format(
    RANKER_LIST_ID, RANKER_ITEM_ID
)
REQUEST_INTERVAL = 5 * 60  # seconds


firebase_admin.initialize_app()
db = firestore.client()

csv_writer = csv.writer(sys.stdout)
db_collection = db.collection("ranker").document(RANKER_LIST_ID).collection("stats")

while True:
    t = time.time()

    data = requests.get(API_URL).json()
    votes_data, reranks_data = data["votes"], data["crowdRankedStats"]

    timestamp = datetime.utcfromtimestamp(t)
    rank, upvotes, downvotes, reranks, top5Reranks = (
        data["rank"],
        votes_data["upVotes"],
        votes_data["downVotes"],
        reranks_data["totalContributingListCount"],
        reranks_data["top5ListCount"],
    )

    csv_writer.writerow(
        [
            timestamp.strftime("%Y-%m-%d %H:%M:%S"),
            rank,
            upvotes,
            downvotes,
            reranks,
            top5Reranks,
        ]
    )
    sys.stdout.flush()

    db_collection.add(
        {
            "timestamp": timestamp,
            "rank": rank,
            "upvotes": upvotes,
            "downvotes": downvotes,
            "reranks": reranks,
            "top5Reranks": top5Reranks,
        }
    )

    while time.time() - t < REQUEST_INTERVAL:
        time.sleep(1)
