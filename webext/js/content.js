(async function() {
  'use strict';

  const Chars = "-a-zA-Z#'";
  const CharPattern = new RegExp('[' + Chars + ']');
  const SplitPattern = new RegExp('[' + Chars + ']+|[^' + Chars + ']+', 'g');


  let values = await browser.storage.local.get();
  let EndPoint = values.apiEndPoint || 'http://127.0.0.1:8116';


  function install() {
    let lastWord;
    let selectedWord;
    let selectionDelayTimer;

    document.body.addEventListener('mousemove', ev => {
      if (selectedWord)
        return;

      let word = extractWordOnCursor(ev.target, ev.clientX, ev.clientY);

      if (word && (lastWord !== word) && CharPattern.test(word))
        request(word);
    });

    document.addEventListener('selectionchange', onSelectionChange);

    function onSelectionChange() {
      if (selectionDelayTimer)
        clearTimeout(selectionDelayTimer);

      selectionDelayTimer = setTimeout(function() {
        selectionDelayTimer = null;

        let selection = window.getSelection().toString();
        selectedWord = selection && selection.trim();
        request(selectedWord);
      }, 100);
    }

    function request(word) {
      lastWord = word;
      fetch(EndPoint + '/word/' + encodeURIComponent(word));
    }

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
      let result = [];
      let size = 0;

      words.forEach(word => {
        count += word.length;
        if (offset <= count && size < 5) {
          result.push(word);
          if (CharPattern.test(word))
            size++;
        }
      });

      return result.join('').replace(/\s+/g, ' ');
    }
  }

  function tryToInstall() {
    fetch(EndPoint + '/ack').then(async resp => {
      let text = await resp.text();
      if (text == '␆')
        install();
    }).catch(error => {
      function onSelectionChange() {
        document.removeEventListener('selectionchange', onSelectionChange);
        setTimeout(tryToInstall, 500);
      }
      document.addEventListener('selectionchange', onSelectionChange);
    });
  }


  tryToInstall();

})();

