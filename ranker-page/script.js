const RANKER_API_URL = 'https://api.ranker.com/lists/298553/items/85372114';
const ITEMS_PER_PAGE = 25;

fetch(RANKER_API_URL)
  .then(response => response.json())
  .then(json => {
    const rank = json.rank;
    const page = Math.ceil(rank / ITEMS_PER_PAGE);
    document.getElementById('loading').style.display = 'none';
    document.getElementById('content').style.display = 'initial';
    document.getElementById('rank').textContent = rank;
    document.getElementById('page').textContent = page;
    document.getElementById('pageLink').href =
      'https://www.ranker.com/crowdranked-list/the-best-movies-of-all-time?page=' +
      page;
  })
  .catch(error => {
    document.getElementById('loading').style.display = 'none';
    document.getElementById('error').style.display = 'initial';
    console.error(error);
  });
