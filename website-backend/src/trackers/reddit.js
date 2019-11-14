const axios = require('axios');
const typeCheck = require('../utils/typeCheck');

const REDDIT_API = 'https://api.reddit.com';
const USER_AGENT = 'subreddit subscriber count tracker v3.0 (by /u/dmitmel)';

module.exports = {
  name: 'reddit',
  createFetcher: ({ subreddits }) => {
    typeCheck.assert(subreddits, 'subreddits', 'Array');
    subreddits.forEach(subreddit =>
      typeCheck.assert(subreddit, 'subreddit', 'String'),
    );

    return () =>
      Promise.all(
        subreddits.map(subreddit =>
          axios
            .get(`${REDDIT_API}/r/${subreddit}/about`, {
              headers: {
                'User-Agent': USER_AGENT,
              },
            })
            .then(({ data }) => {
              typeCheck.assert(data, 'data', 'Object');
              let subredditData = data.data;
              typeCheck.assert(subredditData, 'subredditData', 'Object');

              let {
                subscribers,
                accounts_active: accountsActive,
              } = subredditData;
              typeCheck.assert(subscribers, 'subscribers', 'Number');
              typeCheck.assert(accountsActive, 'accountsActive', 'Number');

              return { name: subreddit, subscribers, accountsActive };
            }),
        ),
      );
  },
};
