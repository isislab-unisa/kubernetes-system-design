(function() {
  if (!window.location.pathname.endsWith('/print.html')) return;

  document.addEventListener('DOMContentLoaded', function () {
    var content = document.getElementById('mdbook-content');
    if (!content) return;

    var headings = content.querySelectorAll('h1, h2, h3');
    if (headings.length === 0) return;

    var style = document.createElement('style');
    style.textContent =
      '#print-toc { break-after: page; padding: 2em 0; }' +
      '#print-toc > h1 { margin-bottom: 1.5em; border-bottom: none; }' +
      '#print-toc ul { list-style: none; padding: 0; margin: 0; }' +
      '#print-toc li { border-bottom: none; padding: 0.2em 0; }' +
      '#print-toc li:last-child { border-bottom: none; }' +
      '#print-toc a { text-decoration: none; color: var(--fg); display: block; }' +
      '#print-toc a:hover { color: var(--links); }' +
      '#print-toc .toc-h1 { font-weight: bold; margin-top: 0em; }' +
      '#print-toc .toc-h2 { padding-left: 1.5em; margin-top: 0em; }' +
      '#print-toc .toc-h3 { padding-left: 3em; opacity: 0.8; margin-top: 0em; }';
    document.head.appendChild(style);

    var toc = document.createElement('div');
    toc.id = 'print-toc';
    toc.innerHTML = '<h1>Table of contents</h1>';

    var list = document.createElement('ul');

    headings.forEach(function (heading) {
      if (!heading.id) return;

      var tag = heading.tagName.toLowerCase();
      var li = document.createElement('li');
      li.className = 'toc-' + tag;

      var a = document.createElement('a');
      a.href = '#' + heading.id;
      a.textContent = heading.textContent;

      li.appendChild(a);
      list.appendChild(li);
    });

    toc.appendChild(list);

    var main = content.querySelector('main');
    if (main) {
      var cover = main.querySelector('.print-cover-page');
      if (cover && cover.parentNode === main) {
        main.insertBefore(toc, cover.nextSibling);
      } else {
        main.insertBefore(toc, main.firstChild);
      }
    }
  });
})();
