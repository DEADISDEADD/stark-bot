import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface MarkdownRendererProps {
  content: string;
}

export default function MarkdownRenderer({ content }: MarkdownRendererProps) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        h1: ({ children }) => (
          <h1 className="text-3xl font-bold text-slate-100 mb-6 pb-2 border-b border-slate-700">
            {children}
          </h1>
        ),
        h2: ({ children }) => (
          <h2 className="text-2xl font-semibold text-slate-200 mt-8 mb-4">
            {children}
          </h2>
        ),
        h3: ({ children }) => (
          <h3 className="text-xl font-semibold text-slate-300 mt-6 mb-3">
            {children}
          </h3>
        ),
        h4: ({ children }) => (
          <h4 className="text-lg font-medium text-slate-300 mt-4 mb-2">
            {children}
          </h4>
        ),
        p: ({ children }) => (
          <p className="text-slate-300 mb-4 leading-relaxed">{children}</p>
        ),
        ul: ({ children }) => (
          <ul className="list-disc list-inside mb-4 space-y-1 text-slate-300 ml-4">
            {children}
          </ul>
        ),
        ol: ({ children }) => (
          <ol className="list-decimal list-inside mb-4 space-y-1 text-slate-300 ml-4">
            {children}
          </ol>
        ),
        li: ({ children }) => <li className="text-slate-300">{children}</li>,
        a: ({ href, children }) => (
          <a
            href={href}
            className="text-stark-400 hover:text-stark-300 underline"
            target={href?.startsWith('http') ? '_blank' : undefined}
            rel={href?.startsWith('http') ? 'noopener noreferrer' : undefined}
          >
            {children}
          </a>
        ),
        code: ({ className, children }) => {
          const isBlock = className?.includes('language-');
          if (isBlock) {
            return (
              <code className="block bg-slate-900 rounded-lg p-4 overflow-x-auto text-sm text-slate-300 font-mono mb-4">
                {children}
              </code>
            );
          }
          return (
            <code className="bg-slate-700 px-1.5 py-0.5 rounded text-sm text-stark-300 font-mono">
              {children}
            </code>
          );
        },
        pre: ({ children }) => (
          <pre className="bg-slate-900 rounded-lg p-4 overflow-x-auto mb-4">
            {children}
          </pre>
        ),
        blockquote: ({ children }) => (
          <blockquote className="border-l-4 border-stark-500 pl-4 italic text-slate-400 my-4">
            {children}
          </blockquote>
        ),
        table: ({ children }) => (
          <div className="overflow-x-auto mb-4">
            <table className="w-full border-collapse border border-slate-700">
              {children}
            </table>
          </div>
        ),
        th: ({ children }) => (
          <th className="border border-slate-700 px-4 py-2 bg-slate-800 text-left text-slate-200 font-semibold">
            {children}
          </th>
        ),
        td: ({ children }) => (
          <td className="border border-slate-700 px-4 py-2 text-slate-300">
            {children}
          </td>
        ),
        hr: () => <hr className="border-slate-700 my-8" />,
        strong: ({ children }) => (
          <strong className="font-semibold text-slate-200">{children}</strong>
        ),
        em: ({ children }) => <em className="italic text-slate-300">{children}</em>,
      }}
    >
      {content}
    </ReactMarkdown>
  );
}
