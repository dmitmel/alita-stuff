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


def reddit_api_request(url):
    response = requests.get(
        REDDIT_API + url,
        headers={
            "User-Agent": "subreddit subscriber count tracker v2.0 (by /u/dmitmel)"
        },
    )
    return response.json()


def fetch_subreddit_id(name):
    return reddit_api_request("/r/" + name + "/about")["data"]["name"]


SUBREDDIT_IDS = [fetch_subreddit_id(name) for name in SUBREDDITS]


def fetch_data():
    timestamp = datetime.utcnow()
    data = reddit_api_request("/api/info?id=" + ",".join(SUBREDDIT_IDS))["data"]
    subscribers = [
        subreddit_data["data"]["subscribers"] for subreddit_data in data["children"]
    ]
    return (timestamp, *subscribers)


csv_writer = csv.writer(sys.stdout)

while True:
    t = time.time()

    row = fetch_data()
    timestamp, *data = row
    csv_writer.writerow([timestamp.strftime("%Y-%m-%d %H:%M:%S")] + data)
    sys.stdout.flush()

    while time.time() - t < REQUEST_INTERVAL:
        time.sleep(1)
