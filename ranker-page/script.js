window.addEventListener('load', function() {
  function getById(id) {
    return document.getElementById(id);
  }

  function showSection(idToShow) {
    ['loading', 'stats', 'error'].forEach(sectionId => {
      var section = getById(sectionId);
      section.style.display = sectionId === idToShow ? '' : 'none';
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
      getById('rank').textContent = rank;
      getById('page').textContent = page;
      getById('pageLink').href =
        'https://www.ranker.com/crowdranked-list/the-best-movies-of-all-time?page=' +
        page;

      var upvotes = json.votes.upVotes;
      var downvotes = json.votes.downVotes;
      var allVotes = upvotes + downvotes;
      getById('upvotes').textContent = upvotes;
      getById('upvotesPercent').textContent = percent(upvotes / allVotes);
      getById('downvotes').textContent = downvotes;
      getById('downvotesPercent').textContent = percent(downvotes / allVotes);

      var top5Reranks = json.crowdRankedStats.top5ListCount;
      var reranks = json.crowdRankedStats.totalContributingListCount;
      getById('top5Reranks').textContent = top5Reranks;
      getById('reranks').textContent = reranks;
      getById('top5ReranksPercent').textContent = percent(
        top5Reranks / reranks,
      );
    })
    .catch(function(error) {
      showSection('error');
      console.error(error);
    });
});
