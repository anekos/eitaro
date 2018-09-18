(function() {
  'use strict';

  const Chars = "a-zA-Z#-'";
  const CharPattern = new RegExp('[' + Chars + ']');
  const SplitPattern = new RegExp('[' + Chars + ']+|[^' + Chars + ']+', 'g');
  const EndPoint = 'http://127.0.0.1:8116'

  function main() {
    let lastWord;

    document.body.addEventListener('mousemove', ev => {
      const word = extractWordOnCursor(ev.target, ev.clientX, ev.clientY);
      if (!word || (lastWord == word) || !CharPattern.test(word))
        return;
      fetch(EndPoint + '/word/' + encodeURIComponent(word));
      lastWord = word;
    });

    function extractWordOnCursor(element, x, y) {
      const caretPosition = element.ownerDocument.caretPositionFromPoint(x, y);

      let caretNode = caretPosition.offsetNode;
      if (caretNode.nodeType !== Node.TEXT_NODE) {
        return;
      }

      let text = caretNode.data;
      let offset = caretPosition.offset;

      let words = text.match(SplitPattern);
      let count = 0;
      let result;

      words.some(word => {
        count += word.length;
        let found = offset <= count;
        if (found)
          result = word;
        return found;
      });

      return result;
    }
  }

  fetch(EndPoint + '/ack').then(async resp => {
    let text = await resp.text();
    if (text == '␆')
      main();
  });

})();

