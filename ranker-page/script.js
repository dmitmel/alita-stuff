$(function() {
  function showSection(idToShow) {
    ['loading', 'stats', 'error'].forEach(sectionId => {
      var section = $('#' + sectionId);
      section.toggle(sectionId === idToShow);
    });
  }

  var RANKER_API_URL =
    'https://api.ranker.com/lists/298553/items/85372114?include=votes,crowdRankedStats';
  var ITEMS_PER_PAGE = 25;

  function percent(value) {
    return (value * 100).toFixed(2);
  }

  showSection('loading');
  fetch(RANKER_API_URL)
    .then(function(response) {
      return response.json();
    })
    .then(function(json) {
      console.log('API response', json);
      showSection('stats');

      var rank = json.rank;
      var page = Math.ceil(rank / ITEMS_PER_PAGE);
      $('#rank').text(rank);
      $('#page').text(page);
      $('#pageLink').attr(
        'href',
        'https://www.ranker.com/crowdranked-list/the-best-movies-of-all-time?page=' +
          page,
      );

      var upvotes = json.votes.upVotes;
      var downvotes = json.votes.downVotes;
      var allVotes = upvotes + downvotes;
      $('#upvotes').text(upvotes);
      $('#upvotesPercent').text(percent(upvotes / allVotes));
      $('#downvotes').text(downvotes);
      $('#downvotesPercent').text(percent(downvotes / allVotes));

      var top5Reranks = json.crowdRankedStats.top5ListCount;
      var reranks = json.crowdRankedStats.totalContributingListCount;
      $('#top5Reranks').text(top5Reranks);
      $('#reranks').text(reranks);
      $('#top5ReranksPercent').text(percent(top5Reranks / reranks));
    })
    .catch(function(error) {
      showSection('error');
      console.error(error);
    });
});
