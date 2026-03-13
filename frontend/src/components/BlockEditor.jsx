// Block Editor Component
import { useState } from 'react';
import { blocks as blocksApi } from '../api';

const BLOCK_TYPES = [
  { value: 'HeroHeader', label: 'Hero Header', icon: '🎯' },
  { value: 'RichText', label: 'Rich Text', icon: '📝' },
  { value: 'ProjectGrid', label: 'Project Grid', icon: '🗂️' },
  { value: 'SkillMatrix', label: 'Skill Matrix', icon: '⚡' },
  { value: 'ContactForm', label: 'Contact Form', icon: '📧' },
  { value: 'TestimonialSlider', label: 'Testimonials', icon: '💬' },
];

// Default content for each block type
const DEFAULT_CONTENT = {
  HeroHeader: { title: 'Welcome', subtitle: 'Your subtitle here', backgroundImage: '' },
  RichText: { html: '<p>Your content here...</p>' },
  ProjectGrid: { title: 'My Projects', items: [] },
  SkillMatrix: { skills: [] },
  ContactForm: { fields: ['name', 'email', 'message'], recipients: [] },
  TestimonialSlider: { testimonials: [] },
};

export function BlockEditor({ pageId, blocks: initialBlocks = [], onBlocksChange }) {
  const [blocks, setBlocks] = useState(initialBlocks);
  const [editingBlock, setEditingBlock] = useState(null);
  const [showAddMenu, setShowAddMenu] = useState(false);

  const addBlock = async (type) => {
    try {
      const res = await blocksApi.create({
        page_id: pageId,
        block_type: type,
        title: '',
        content: DEFAULT_CONTENT[type] || {},
      });
      const newBlocks = [...blocks, res.data];
      setBlocks(newBlocks);
      onBlocksChange?.(newBlocks);
      setShowAddMenu(false);
    } catch (err) {
      console.error('Failed to add block:', err);
    }
  };

  const updateBlock = async (id, updates) => {
    try {
      const res = await blocksApi.update(id, updates);
      const newBlocks = blocks.map(b => b.id === id ? res.data : b);
      setBlocks(newBlocks);
      onBlocksChange?.(newBlocks);
      setEditingBlock(null);
    } catch (err) {
      console.error('Failed to update block:', err);
    }
  };

  const deleteBlock = async (id) => {
    if (!confirm('Delete this block?')) return;
    try {
      await blocksApi.delete(id);
      const newBlocks = blocks.filter(b => b.id !== id);
      setBlocks(newBlocks);
      onBlocksChange?.(newBlocks);
    } catch (err) {
      console.error('Failed to delete block:', err);
    }
  };

  const moveBlock = (index, direction) => {
    const newBlocks = [...blocks];
    const targetIndex = index + direction;
    if (targetIndex < 0 || targetIndex >= newBlocks.length) return;
    
    [newBlocks[index], newBlocks[targetIndex]] = [newBlocks[targetIndex], newBlocks[index]];
    setBlocks(newBlocks);
    
    // Save new order
    blocksApi.reorder(newBlocks.map((b, i) => ({ id: b.id, order_index: i })))
      .then(res => setBlocks(res.data))
      .catch(console.error);
  };

  return (
    <div className="block-editor">
      <div className="blocks-list">
        {blocks.map((block, index) => (
          <div key={block.id} className={`block-item block-${block.block_type?.toLowerCase()}`}>
            <div className="block-header">
              <span className="block-type">{block.block_type}</span>
              <div className="block-actions">
                <button onClick={() => moveBlock(index, -1)} disabled={index === 0}>↑</button>
                <button onClick={() => moveBlock(index, 1)} disabled={index === blocks.length - 1}>↓</button>
                <button onClick={() => setEditingBlock(block)}>Edit</button>
                <button onClick={() => deleteBlock(block.id)} className="btn-danger">Delete</button>
              </div>
            </div>
            <div className="block-preview">
              {renderBlockPreview(block)}
            </div>
          </div>
        ))}
        
        <div className="add-block-menu">
          <button onClick={() => setShowAddMenu(!showAddMenu)} className="btn-primary">
            + Add Block
          </button>
          {showAddMenu && (
            <div className="block-type-menu">
              {BLOCK_TYPES.map(type => (
                <button key={type.value} onClick={() => addBlock(type.value)}>
                  {type.icon} {type.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {editingBlock && (
        <BlockForm 
          block={editingBlock} 
          onSave={updateBlock} 
          onClose={() => setEditingBlock(null)} 
        />
      )}
    </div>
  );
}

function BlockForm({ block, onSave, onClose }) {
  const [title, setTitle] = useState(block.title || '');
  const [content, setContent] = useState(block.content || {});
  const [status, setStatus] = useState(block.status || 'draft');

  const handleSubmit = (e) => {
    e.preventDefault();
    onSave(block.id, { title, content, status });
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h3>Edit {block.block_type}</h3>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Title</label>
            <input value={title} onChange={e => setTitle(e.target.value)} />
          </div>
          
          <div className="form-group">
            <label>Status</label>
            <select value={status} onChange={e => setStatus(e.target.value)}>
              <option value="draft">Draft</option>
              <option value="published">Published</option>
              <option value="archived">Archived</option>
            </select>
          </div>

          <ContentEditor type={block.block_type} content={content} onChange={setContent} />

          <div className="form-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  );
}

function ContentEditor({ type, content, onChange }) {
  switch (type) {
    case 'HeroHeader':
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
    case 'RichText':
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
    case 'ProjectGrid':
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
    case 'SkillMatrix':
      return (
        <div className="form-group">
          <label>Skills (comma separated)</label>
          <input 
            value={(content.skills || []).join(', ')} 
            onChange={e => onChange({ ...content, skills: e.target.value.split(',').map(s => s.trim()).filter(Boolean) })} 
          />
        </div>
      );
    default:
      return (
        <div className="form-group">
          <label>JSON Content</label>
          <textarea 
            value={JSON.stringify(content, null, 2)} 
            onChange={e => {
              try { onChange(JSON.parse(e.target.value)); } catch {}
            }} 
            rows={6}
          />
        </div>
      );
  }
}

function renderBlockPreview(block) {
  const { block_type, title, content } = block;
  switch (block_type) {
    case 'HeroHeader':
      return <div className="preview-hero"><h3>{title || content?.title}</h3><p>{content?.subtitle}</p></div>;
    case 'RichText':
      return <div className="preview-text" dangerouslySetInnerHTML={{ __html: content?.html }} />;
    case 'ProjectGrid':
      return <div className="preview-grid">{content?.items?.length || 0} projects</div>;
    case 'SkillMatrix':
      return <div className="preview-skills">{(content?.skills || []).join(', ')}</div>;
    default:
      return <div className="preview-generic">{block_type}</div>;
  }
}

export default BlockEditor;
