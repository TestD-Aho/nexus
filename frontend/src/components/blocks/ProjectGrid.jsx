import './ProjectGrid.css';

export function ProjectGridView({ block }) {
  const { title, content } = block;
  return (
    <section className="block-projects">
      <h2>{title || content?.title || 'Projects'}</h2>
      <div className="projects-grid">
        {(content?.items || []).map((item, i) => (
          <div key={i} className="project-card">
            <h3>{item.title}</h3>
            <p>{item.description}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

export function ProjectGridEdit({ content, onChange }) {
  return (
    <div className="form-group">
      <label>Projects (one per line: title|description)</label>
      <textarea 
        value={(content.items || []).map(i => `${i.title}|${i.description}`).join('\n')}
        onChange={e => onChange({ 
          ...content, 
          items: e.target.value.split('\n').filter(Boolean).map(line => {
            const [title, description] = line.split('|');
            return { title, description };
          })
        })} 
        rows={6}
      />
    </div>
  );
}