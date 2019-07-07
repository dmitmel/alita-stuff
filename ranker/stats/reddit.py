#!/usr/bin/env python3

# usage: ./reddit.py | tee -a reddit_stats.csv

import time
from datetime import datetime
import sys
import csv
import requests

SUBREDDITS = ["alitabattleangel", "Gunnm"]
REDDIT_API = "https://api.reddit.com"
REQUEST_INTERVAL = 10 * 60  # seconds


def fetch_data():
    timestamp = datetime.utcnow()
    subscribers = []
    accounts_active = []
    for subreddit in SUBREDDITS:
        response = requests.get(
            REDDIT_API + "/r/" + subreddit + "/about",
            headers={
                "User-Agent": "subreddit subscriber count tracker v2.0 (by /u/dmitmel)"
            },
        )
        subreddit_data = response.json()["data"]
        subscribers.append(subreddit_data["subscribers"])
        accounts_active.append(subreddit_data["accounts_active"])
    return (timestamp, *subscribers, *accounts_active)


csv_writer = csv.writer(sys.stdout)

while True:
    t = time.time()

    row = fetch_data()
    timestamp, *data = row
    csv_writer.writerow([timestamp.strftime("%Y-%m-%d %H:%M:%S")] + data)
    sys.stdout.flush()

    while time.time() - t < REQUEST_INTERVAL:
        time.sleep(1)
