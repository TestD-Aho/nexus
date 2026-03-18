// Block Editor Component
import { useState } from 'react';
import { blocks as blocksApi } from '../api';
import { BLOCK_REGISTRY } from './blocks/registry';

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
        content: BLOCK_REGISTRY[type]?.defaultContent || {},
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
    if (!window.confirm('Delete this block?')) return;
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
        {blocks.map((block, index) => {
          const registryEntry = BLOCK_REGISTRY[block.block_type];
          return (
            <div key={block.id} className={`block-item block-${block.block_type?.toLowerCase()}`}>
              <div className="block-header">
                <span className="block-type">{registryEntry?.icon} {registryEntry?.label || block.block_type}</span>
                <div className="block-actions">
                  <button onClick={() => moveBlock(index, -1)} disabled={index === 0}>↑</button>
                  <button onClick={() => moveBlock(index, 1)} disabled={index === blocks.length - 1}>↓</button>
                  <button onClick={() => setEditingBlock(block)}>Edit</button>
                  <button onClick={() => deleteBlock(block.id)} className="btn-danger">Delete</button>
                </div>
              </div>
              <div className="block-preview">
                {registryEntry ? registryEntry.preview(block) : <div className="preview-generic">{block.block_type}</div>}
              </div>
            </div>
          );
        })}
        
        <div className="add-block-menu">
          <button onClick={() => setShowAddMenu(!showAddMenu)} className="btn-primary">
            + Add Block
          </button>
          {showAddMenu && (
            <div className="block-type-menu">
              {Object.entries(BLOCK_REGISTRY).map(([type, config]) => (
                <button key={type} onClick={() => addBlock(type)}>
                  {config.icon} {config.label}
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

  const registryEntry = BLOCK_REGISTRY[block.block_type];
  const EditorComponent = registryEntry?.Edit;

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h3>Edit {registryEntry?.label || block.block_type}</h3>
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

          {EditorComponent ? (
            <EditorComponent content={content} onChange={setContent} />
          ) : (
            <div className="form-group">
              <label>JSON Content</label>
              <textarea 
                value={JSON.stringify(content, null, 2)} 
                onChange={e => {
                  try { setContent(JSON.parse(e.target.value)); } catch {}
                }} 
                rows={6}
              />
            </div>
          )}

          <div className="form-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default BlockEditor;