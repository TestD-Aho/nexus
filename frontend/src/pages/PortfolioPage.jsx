// Portfolio Page
import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { projects, system } from '../api';
import ProjectCard from '../components/ProjectCard';

export default function PortfolioPage() {
  const [projectsList, setProjectsList] = useState([]);
  const [cvUrl, setCvUrl] = useState('');
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('all'); // 'all' or 'featured'
  const [search, setSearch] = useState('');

  useEffect(() => {
    loadProjects();
    loadSettings();
  }, [filter, search]);

  const loadSettings = async () => {
    try {
      const res = await system.settings();
      if (res.data.cv_url) {
        setCvUrl(res.data.cv_url);
      }
    } catch (err) {
      console.error('Failed to load settings:', err);
    }
  };

  const loadProjects = async () => {
    setLoading(true);
    try {
      const params = {};
      if (filter === 'featured') {
        params.featured = true;
      }
      if (search) {
        params.search = search;
      }
      const res = await projects.list(params);
      setProjectsList(res.data);
    } catch (err) {
      console.error('Failed to load projects:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <div className="loading">Loading projects...</div>;

  return (
    <div className="portfolio-page">
      <div className="page-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h1>Portfolio</h1>
        {cvUrl && (
          <a href={cvUrl} target="_blank" rel="noreferrer" className="btn btn-primary" download>
            📄 Download CV
          </a>
        )}
      </div>

      <div className="filter-and-search">
        <div className="filter-tabs">
          <button 
            className={filter === 'all' ? 'active' : ''} 
            onClick={() => setFilter('all')}
          >All</button>
          <button 
            className={filter === 'featured' ? 'active' : ''} 
            onClick={() => setFilter('featured')}
          >Featured</button>
        </div>
        <div className="search-box">
          <input
            type="text"
            placeholder="Search projects..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
      </div>

      <div className="projects-grid">
        {projectsList.length === 0 ? (
          <div className="empty-state">
            <p>No projects yet.</p>
          </div>
        ) : (
          projectsList.map(project => (
            <ProjectCard key={project.id} project={project} />
          ))
        )}
      </div>
    </div>
  );
}