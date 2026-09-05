// Minimal Markdown → HTML preview renderer. Deliberately dependency-free for
// the prototype: headings, paragraphs, fenced code, inline code, bold/italic,
// unordered lists, blockquotes and rules. Input is HTML-escaped *before*
// transforms, so node bodies can never inject markup.

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function inline(text: string): string {
  return text
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>')
}

function escapeAndInline(line: string): string {
  return inline(escapeHtml(line))
}

function renderInline(line: string): string {
  return escapeAndInline(line)
}

/** Render Markdown body text (no frontmatter) to an HTML fragment. */
export function mdToHtml(md: string): string {
  const rawLines = md.replace(/\r\n/g, '\n').split('\n')
  const out: string[] = []

  let i = 0
  const pushBlock = (tag: string, lines: string[], cls = '') => {
    if (!lines.length) return
    const inner = lines.map((l) => renderInline(l.trim())).join('\n')
    out.push(`<${tag}${cls ? ` class="${cls}"` : ''}>${inner}</${tag}>`)
  }

  while (i < rawLines.length) {
    const line = rawLines[i]

    // Fenced code block
    if (/^```/.test(line)) {
      const buf: string[] = []
      i++
      while (i < rawLines.length && !/^```/.test(rawLines[i])) {
        buf.push(escapeHtml(rawLines[i]))
        i++
      }
      i++ // closing fence
      out.push(`<pre><code>${buf.join('\n')}</code></pre>`)
      continue
    }

    // Headings
    const h = /^(#{1,6})\s+(.*)$/.exec(line)
    if (h) {
      out.push(`<h${h[1].length}>${renderInline(h[2])}</h${h[1].length}>`)
      i++
      continue
    }

    // Horizontal rule: three or more of the same marker, spaces allowed between
    if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(line)) {
      out.push('<hr />')
      i++
      continue
    }

    // Unordered list
    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = []
      while (i < rawLines.length && /^\s*[-*+]\s+/.test(rawLines[i])) {
        items.push(renderInline(rawLines[i].replace(/^\s*[-*+]\s+/, '')))
        i++
      }
      out.push(`<ul>${items.map((it) => `<li>${it}</li>`).join('')}</ul>`)
      continue
    }

    // Blockquote
    if (/^\s*>\s?/.test(line)) {
      const quotes: string[] = []
      while (i < rawLines.length && /^\s*>\s?/.test(rawLines[i])) {
        quotes.push(renderInline(rawLines[i].replace(/^\s*>\s?/, '')))
        i++
      }
      out.push(`<blockquote>${quotes.join('<br />')}</blockquote>`)
      continue
    }

    // Blank line: skip
    if (line.trim() === '') {
      i++
      continue
    }

    // Paragraph: consume until a blank line or another block opener
    const buf: string[] = []
    while (
      i < rawLines.length &&
      rawLines[i].trim() !== '' &&
      !/^(#{1,6})\s+/.test(rawLines[i]) &&
      !/^```/.test(rawLines[i]) &&
      !/^\s*[-*+]\s+/.test(rawLines[i]) &&
      !/^\s*>\s?/.test(rawLines[i])
    ) {
      buf.push(rawLines[i])
      i++
    }
    pushBlock('p', buf)
  }
  return out.join('\n')
}
