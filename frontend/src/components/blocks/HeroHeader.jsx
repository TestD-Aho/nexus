import './HeroHeader.css';

export function HeroHeaderView({ block }) {
  const { title, content } = block;
  return (
    <section 
      className="block-hero" 
      style={{ backgroundImage: content?.backgroundImage ? `url(${content.backgroundImage})` : undefined }}
    >
      <div className="hero-content">
        <h1>{title || content?.title}</h1>
        <p>{content?.subtitle}</p>
      </div>
    </section>
  );
}

export function HeroHeaderEdit({ content, onChange }) {
  return (
    <>
      <div className="form-group">
        <label>Subtitle</label>
        <input 
          value={content.subtitle || ''} 
          onChange={e => onChange({ ...content, subtitle: e.target.value })} 
        />
      </div>
      <div className="form-group">
        <label>Background Image URL</label>
        <input 
          value={content.backgroundImage || ''} 
          onChange={e => onChange({ ...content, backgroundImage: e.target.value })} 
        />
      </div>
    </>
  );
}