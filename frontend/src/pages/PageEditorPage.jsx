// Page Editor (Create/Edit)
import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { pages, blocks as blocksApi } from '../api';
import { BlockEditor } from '../components/BlockEditor';

export function PageEditorPage() {
  const { slug } = useParams();
  const navigate = useNavigate();
  const isNew = slug === 'new';

  const [page, setPage] = useState(null);
  const [blocks, setBlocks] = useState([]);
  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    slug: '',
    title: '',
    description: '',
    is_published: false,
    is_home: false,
    meta_title: '',
    meta_description: '',
  });

  useEffect(() => {
    if (!isNew) {
      loadPage();
    }
  }, [slug]);

  const loadPage = async () => {
    try {
      const [pageRes, blocksRes] = await Promise.all([
        pages.get(slug),
        blocksApi.list({ page_id: slug }), // Note: should use page ID
      ]);
      setPage(pageRes.data.page);
      setBlocks(blocksRes.data);
      setForm({
        slug: pageRes.data.page.slug,
        title: pageRes.data.page.title,
        description: pageRes.data.page.description || '',
        is_published: pageRes.data.page.is_published,
        is_home: pageRes.data.page.is_home,
        meta_title: pageRes.data.page.meta_title || '',
        meta_description: pageRes.data.page.meta_description || '',
      });
    } catch (err) {
      console.error('Failed to load page:', err);
      navigate('/admin');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (e) => {
    e.preventDefault();
    setSaving(true);
    try {
      let pageId;
      if (isNew) {
        const res = await pages.create(form);
        pageId = res.data.id;
        navigate(`/admin/page/${form.slug}`, { replace: true });
      } else {
        await pages.update(page.id, form);
        pageId = page.id;
      }
      setPage({ ...page, ...form, id: pageId });
    } catch (err) {
      console.error('Failed to save:', err);
      alert('Failed to save page');
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm('Delete this page? This cannot be undone.')) return;
    try {
      await pages.delete(page.id);
      navigate('/admin');
    } catch (err) {
      console.error('Delete failed:', err);
    }
  };

  if (loading) return <div className="loading">Loading...</div>;

  return (
    <div className="page-editor">
      <div className="editor-header">
        <Link to="/admin" className="back-link">← Back to Admin</Link>
        <h1>{isNew ? 'Create Page' : 'Edit Page'}</h1>
      </div>

      <form onSubmit={handleSave} className="page-form">
        <div className="form-row">
          <div className="form-group flex-2">
            <label>Title</label>
            <input
              value={form.title}
              onChange={e => setForm({ ...form, title: e.target.value })}
              placeholder="Page Title"
              required
            />
          </div>
          <div className="form-group flex-1">
            <label>Slug</label>
            <input
              value={form.slug}
              onChange={e => setForm({ ...form, slug: e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-') })}
              placeholder="page-slug"
              required
            />
          </div>
        </div>

        <div className="form-group">
          <label>Description</label>
          <input
            value={form.description}
            onChange={e => setForm({ ...form, description: e.target.value })}
            placeholder="Brief description"
          />
        </div>

        <div className="form-row">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={form.is_published}
              onChange={e => setForm({ ...form, is_published: e.target.checked })}
            />
            Published
          </label>
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={form.is_home}
              onChange={e => setForm({ ...form, is_home: e.target.checked })}
            />
            Homepage
          </label>
        </div>

        <div className="form-group">
          <label>Meta Title</label>
          <input
            value={form.meta_title}
            onChange={e => setForm({ ...form, meta_title: e.target.value })}
            placeholder="SEO Title"
          />
        </div>

        <div className="form-group">
          <label>Meta Description</label>
          <textarea
            value={form.meta_description}
            onChange={e => setForm({ ...form, meta_description: e.target.value })}
            placeholder="SEO Description"
            rows={2}
          />
        </div>

        <div className="form-actions">
          <button type="submit" className="btn-primary" disabled={saving}>
            {saving ? 'Saving...' : 'Save Page'}
          </button>
          {!isNew && (
            <button type="button" className="btn-danger" onClick={handleDelete}>
              Delete Page
            </button>
          )}
        </div>
      </form>

      {!isNew && page && (
        <div className="blocks-section">
          <h2>Blocks</h2>
          <BlockEditor
            pageId={page.id}
            blocks={blocks}
            onBlocksChange={setBlocks}
          />
        </div>
      )}
    </div>
  );
}
