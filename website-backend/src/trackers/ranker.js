const axios = require('axios');

module.exports = {
  name: 'ranker',
  createFetcher: ({ listId, itemId }) => {
    let apiUrl = `https://api.ranker.com/lists/${listId}/items/${itemId}?include=crowdRankedStats,votes`;
    return () =>
      axios.get(apiUrl).then(res => ({
        rank: res.data.rank,
        upvotes: res.data.votes.upVotes,
        downvotes: res.data.votes.downVotes,
        reranks: res.data.crowdRankedStats.totalContributingListCount,
        top5Reranks: res.data.crowdRankedStats.top5ListCount,
      }));
  },
};
