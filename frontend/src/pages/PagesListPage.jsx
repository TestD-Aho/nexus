// Pages List Page
import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { pages } from '../api';

export function PagesListPage() {
  const [pagesList, setPagesList] = useState([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('all');

  useEffect(() => {
    loadPages();
  }, [filter]);

  const loadPages = async () => {
    setLoading(true);
    try {
      const params = filter === 'published' ? { published: true } : {};
      const res = await pages.list(params);
      setPagesList(res.data);
    } catch (err) {
      console.error('Failed to load pages:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <div className="loading">Loading pages...</div>;

  return (
    <div className="pages-list-page">
      <div className="page-header">
        <h1>Pages</h1>
        <Link to="/admin/page/new" className="btn-primary">+ New Page</Link>
      </div>

      <div className="filter-tabs">
        <button 
          className={filter === 'all' ? 'active' : ''} 
          onClick={() => setFilter('all')}
        >All</button>
        <button 
          className={filter === 'published' ? 'active' : ''} 
          onClick={() => setFilter('published')}
        >Published</button>
        <button 
          className={filter === 'draft' ? 'active' : ''} 
          onClick={() => setFilter('draft')}
        >Drafts</button>
      </div>

      <div className="pages-grid">
        {pagesList.length === 0 ? (
          <div className="empty-state">
            <p>No pages yet. Create your first page!</p>
          </div>
        ) : (
          pagesList.map(page => (
            <Link 
              key={page.id} 
              to={`/page/${page.slug}`}
              className={`page-card ${page.is_published ? 'published' : 'draft'}`}
            >
              <div className="page-card-content">
                <h3>{page.title}</h3>
                <span className="page-slug">/{page.slug}</span>
                {page.description && <p className="page-desc">{page.description}</p>}
              </div>
              <div className="page-card-status">
                <span className={`status-badge ${page.is_published ? 'published' : 'draft'}`}>
                  {page.is_published ? 'Published' : 'Draft'}
                </span>
              </div>
            </Link>
          ))
        )}
      </div>
    </div>
  );
}
