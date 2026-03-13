// Media Uploader Component
import { useState, useRef } from 'react';
import { media as mediaApi } from '../api';

export function MediaUploader({ onUpload, multiple = true }) {
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const fileInputRef = useRef(null);

  const handleFiles = async (files) => {
    if (!files?.length) return;
    
    setUploading(true);
    try {
      const uploaded = [];
      for (const file of files) {
        const formData = new FormData();
        formData.append('file', file);
        const res = await mediaApi.upload(formData);
        uploaded.push(res.data);
      }
      onUpload?.(uploaded);
    } catch (err) {
      console.error('Upload failed:', err);
      alert('Failed to upload files');
    } finally {
      setUploading(false);
    }
  };

  const handleDrop = (e) => {
    e.preventDefault();
    setDragOver(false);
    handleFiles(e.dataTransfer.files);
  };

  const handleChange = (e) => {
    handleFiles(e.target.files);
  };

  return (
    <div 
      className={`media-uploader ${dragOver ? 'drag-over' : ''} ${uploading ? 'uploading' : ''}`}
      onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}
      onClick={() => fileInputRef.current?.click()}
    >
      <input
        ref={fileInputRef}
        type="file"
        multiple={multiple}
        accept="image/*,video/*,.pdf"
        onChange={handleChange}
        style={{ display: 'none' }}
      />
      {uploading ? (
        <div className="uploading">Uploading...</div>
      ) : (
        <div className="upload-prompt">
          <span className="upload-icon">📁</span>
          <p>Drop files here or click to upload</p>
        </div>
      )}
    </div>
  );
}

// Media Gallery Component
export function MediaGallery({ onSelect }) {
  const [media, setMedia] = useState([]);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState([]);

  useState(() => {
    mediaApi.list()
      .then(res => setMedia(res.data))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const toggleSelect = (item) => {
    if (selected.includes(item.id)) {
      setSelected(selected.filter(id => id !== item.id));
    } else {
      setSelected([...selected, item.id]);
    }
  };

  const handleDelete = async (id) => {
    if (!confirm('Delete this file?')) return;
    try {
      await mediaApi.delete(id);
      setMedia(media.filter(m => m.id !== id));
    } catch (err) {
      console.error('Delete failed:', err);
    }
  };

  if (loading) return <div className="loading">Loading media...</div>;

  return (
    <div className="media-gallery">
      <MediaUploader onUpload={(files) => setMedia([...files, ...media])} />
      
      <div className="media-grid">
        {media.map(item => (
          <div 
            key={item.id} 
            className={`media-item ${selected.includes(item.id) ? 'selected' : ''}`}
            onClick={() => onSelect ? onSelect(item) : toggleSelect(item)}
          >
            {item.mime_type?.startsWith('image/') ? (
              <img src={item.url} alt={item.alt_text || item.original_filename} />
            ) : (
              <div className="file-icon">📄</div>
            )}
            <div className="media-info">
              <span className="filename">{item.original_filename}</span>
            </div>
            <button 
              className="delete-btn" 
              onClick={(e) => { e.stopPropagation(); handleDelete(item.id); }}
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
