const fetch = require('node-fetch');
const typeCheck = require('../utils/typeCheck');

module.exports = {
  name: 'ranker',
  createFetcher: ({ listId, itemId }) => {
    typeCheck.assert(listId, 'listId', 'String');
    typeCheck.assert(itemId, 'itemId', 'String');

    let apiUrl = `https://api.ranker.com/lists/${listId}/items/${itemId}?include=crowdRankedStats,votes`;
    return () =>
      fetch(apiUrl)
        .then(response => response.json())
        .then(data => {
          typeCheck.assert(data, 'data', 'Object');
          let { rank, votes, crowdRankedStats } = data;
          typeCheck.assert(rank, 'rank', 'Number');

          typeCheck.assert(votes, 'votes', 'Object');
          let { upVotes, downVotes } = votes;
          typeCheck.assert(upVotes, 'upVotes', 'Number');
          typeCheck.assert(downVotes, 'downVotes', 'Number');

          typeCheck.assert(crowdRankedStats, 'crowdRankedStats', 'Object');
          let { totalContributingListCount, top5ListCount } = crowdRankedStats;
          typeCheck.assert(
            totalContributingListCount,
            'totalContributingListCount',
            'Number',
          );
          typeCheck.assert(top5ListCount, 'top5ListCount', 'Number');

          return {
            rank,
            upvotes: upVotes,
            downvotes: downVotes,
            reranks: totalContributingListCount,
            top5Reranks: top5ListCount,
          };
        });
  },
};
