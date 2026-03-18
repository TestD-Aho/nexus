import './WorkProcess.css';

export function WorkProcessView({ block }) {
  const { title, content } = block;
  return (
    <section className="block-work-process">
      <h2>{title || 'Work Process'}</h2>
      <ol className="work-process-steps">
        {(content?.steps || []).map((step, i) => (
          <li key={i} className="process-step">
            <h3>{step.title}</h3>
            <p>{step.description}</p>
          </li>
        ))}
      </ol>
    </section>
  );
}

export function WorkProcessEdit({ content, onChange }) {
  return (
    <div className="form-group">
      <label>Work Process Steps</label>
      {(content.steps || []).map((step, index) => (
        <div key={index} className="step-editor" style={{ marginBottom: '10px', padding: '10px', border: '1px solid #ccc' }}>
          <input 
            placeholder="Step Title"
            value={step.title || ''} 
            onChange={e => {
              const newSteps = [...(content.steps || [])];
              newSteps[index] = { ...step, title: e.target.value };
              onChange({ ...content, steps: newSteps });
            }} 
            style={{ display: 'block', marginBottom: '5px' }}
          />
          <textarea 
            placeholder="Step Description"
            value={step.description || ''} 
            onChange={e => {
              const newSteps = [...(content.steps || [])];
              newSteps[index] = { ...step, description: e.target.value };
              onChange({ ...content, steps: newSteps });
            }} 
            rows={2}
            style={{ display: 'block', width: '100%', marginBottom: '5px' }}
          />
          <button 
            type="button" 
            onClick={() => {
              const newSteps = [...(content.steps || [])];
              newSteps.splice(index, 1);
              onChange({ ...content, steps: newSteps });
            }}
            className="btn-danger"
          >
            Remove Step
          </button>
        </div>
      ))}
      <button 
        type="button" 
        onClick={() => {
          const newSteps = [...(content.steps || []), { title: '', description: '' }];
          onChange({ ...content, steps: newSteps });
        }}
      >
        + Add Step
      </button>
    </div>
  );
}