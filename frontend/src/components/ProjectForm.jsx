import React, { useState } from 'react';

const ProjectForm = ({ initialData = {}, onSubmit }) => {
  const [formData, setFormData] = useState({
    title: initialData.title || '',
    slug: initialData.slug || '',
    description: initialData.description || '',
    challenge: initialData.challenge || '',
    solution: initialData.solution || '',
    stack: Array.isArray(initialData.stack) ? initialData.stack.join(', ') : initialData.stack || '',
    role: initialData.role || '',
    live_url: initialData.live_url || '',
    repo_url: initialData.repo_url || '',
    media_ids: Array.isArray(initialData.media_ids) ? initialData.media_ids.join(', ') : initialData.media_ids || '',
    technologies: Array.isArray(initialData.technologies) ? initialData.technologies.join(', ') : initialData.technologies || '',
    featured: !!initialData.featured,
    published_at: initialData.published_at ? new Date(initialData.published_at).toISOString().split('T')[0] : '',
  });

  const handleChange = (e) => {
    const { name, value, type, checked } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value,
    }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    // Convert comma-separated strings back to arrays where needed
    const projectData = {
      ...formData,
      stack: formData.stack.split(',').map(s => s.trim()).filter(s => s),
      media_ids: formData.media_ids.split(',').map(s => s.trim()).filter(s => s),
      technologies: formData.technologies.split(',').map(s => s.trim()).filter(s => s),
    };
    onSubmit(projectData);
  };

  return (
    <form onSubmit={handleSubmit} style={{ maxWidth: '600px', margin: '0 auto', padding: '20px' }}>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Title</label>
        <input
          type="text"
          name="title"
          value={formData.title}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
          required
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Slug</label>
        <input
          type="text"
          name="slug"
          value={formData.slug}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
          required
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Description</label>
        <textarea
          name="description"
          value={formData.description}
          onChange={handleChange}
          style={{ width: '100%', height: '80px', padding: '8px', boxSizing: 'border-box' }}
          required
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Challenge</label>
        <textarea
          name="challenge"
          value={formData.challenge}
          onChange={handleChange}
          style={{ width: '100%', height: '80px', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Solution</label>
        <textarea
          name="solution"
          value={formData.solution}
          onChange={handleChange}
          style={{ width: '100%', height: '80px', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Stack (comma-separated)</label>
        <input
          type="text"
          name="stack"
          value={formData.stack}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Role</label>
        <input
          type="text"
          name="role"
          value={formData.role}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Live URL</label>
        <input
          type="url"
          name="live_url"
          value={formData.live_url}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Repo URL</label>
        <input
          type="url"
          name="repo_url"
          value={formData.repo_url}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Media IDs (comma-separated UUIDs)</label>
        <input
          type="text"
          name="media_ids"
          value={formData.media_ids}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Technologies (comma-separated)</label>
        <input
          type="text"
          name="technologies"
          value={formData.technologies}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <div style={{ marginBottom: '16px', display: 'flex', alignItems: 'center' }}>
        <input
          type="checkbox"
          name="featured"
          checked={formData.featured}
          onChange={handleChange}
          style={{ marginRight: '8px' }}
        />
        <label style={{ fontWeight: 'bold', margin: 0 }}>Featured</label>
      </div>
      <div style={{ marginBottom: '16px' }}>
        <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>Published At</label>
        <input
          type="date"
          name="published_at"
          value={formData.published_at}
          onChange={handleChange}
          style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
        />
      </div>
      <button
        type="submit"
        style={{
          backgroundColor: '#007bff',
          color: 'white',
          border: 'none',
          padding: '10px 20px',
          cursor: 'pointer',
          fontSize: '16px',
          borderRadius: '4px',
        }}
      >
        Submit
      </button>
    </form>
  );
};

export default ProjectForm;