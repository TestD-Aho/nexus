import './SkillMatrix.css';

export function SkillMatrixView({ block }) {
  const { title, content } = block;
  return (
    <section className="block-skills">
      <h2>{title || 'Skills'}</h2>
      <div className="skills-list">
        {(content?.skills || []).map((skill, i) => (
          <span key={i} className="skill-tag">{skill}</span>
        ))}
      </div>
    </section>
  );
}

export function SkillMatrixEdit({ content, onChange }) {
  return (
    <div className="form-group">
      <label>Skills (comma separated)</label>
      <input 
        value={(content.skills || []).join(', ')} 
        onChange={e => onChange({ ...content, skills: e.target.value.split(',').map(s => s.trim()).filter(Boolean) })} 
      />
    </div>
  );
}