window.addEventListener('load', function() {
  let countdownDate = new Date(new Date().getFullYear(), 9 - 1, 9, 0, 0, 0);
  if (new Date() > countdownDate)
    countdownDate.setFullYear(countdownDate.getFullYear() + 1);

  let digitsElements = {};
  ['days', 'hours', 'minutes', 'seconds'].forEach(id => {
    digitsElements[id] = document.querySelector('#' + id + '.number > .digits');
  });

  function update() {
    let millisecondsLeft = countdownDate.getTime() - Date.now();
    // Personally, I can't think of an explanation why ceil is used here
    // instead of floor like everywhere else. I determined with some
    // experimentation that this is the right way.
    let secondsLeft = Math.ceil(millisecondsLeft / 1000);
    setNumber('seconds', secondsLeft % 60);
    let minutesLeft = Math.floor(secondsLeft / 60);
    setNumber('minutes', minutesLeft % 60);
    let hoursLeft = Math.floor(minutesLeft / 60);
    setNumber('hours', hoursLeft % 24);
    let daysLeft = Math.floor(hoursLeft / 24);
    setNumber('days', daysLeft);

    requestAnimationFrame(update);
  }

  // basically I have reinvented Virtual DOM here
  function setNumber(id, value) {
    let text = padLeft(value.toString(), 2, '0');
    let digits = digitsElements[id];

    // remove extra elements if needed
    for (let i = digits.childElementCount; i > text.length; i--) {
      digits.removeChild(digits.lastElementChild);
    }

    // add missing elements if needed
    for (let i = digits.childElementCount; i < text.length; i++) {
      let digit = document.createElement('div');
      digit.className = 'digit';
      digits.appendChild(digit);
    }

    for (let i = 0; i < digits.childElementCount; i++) {
      let digit = digits.children[i];
      // only update this digit if needed
      if (text[i] !== digit.textContent) digit.textContent = text[i];
    }
  }

  // ah, the famous left-pad
  function padLeft(string, length, padding) {
    while (string.length < length) string = padding + string;
    return string;
  }

  update();
});
