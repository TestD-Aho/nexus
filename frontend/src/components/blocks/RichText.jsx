import DOMPurify from 'dompurify';
import './RichText.css';

export function RichTextView({ block }) {
  const { content } = block;
  return (
    <section className="block-richtext">
      <div dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(content?.html || '') }} />
    </section>
  );
}

export function RichTextEdit({ content, onChange }) {
  return (
    <div className="form-group">
      <label>HTML Content</label>
      <textarea 
        value={content.html || ''} 
        onChange={e => onChange({ ...content, html: e.target.value })} 
        rows={6}
      />
    </div>
  );
}