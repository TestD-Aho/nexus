import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { projects } from '../api';

export default function ProjectDetailPage() {
  const { slug } = useParams();
  const [project, setProject] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const loadProject = async () => {
      try {
        setLoading(true);
        // Assuming api.projects.get works with slug or id
        const res = await projects.get(slug);
        setProject(res.data);
      } catch (err) {
        console.error('Failed to load project:', err);
        setError('Project not found');
      } finally {
        setLoading(false);
      }
    };
    loadProject();
  }, [slug]);

  if (loading) return <div className="loading">Loading project...</div>;
  if (error || !project) return <div className="error">{error || 'Project not found'}</div>;

  return (
    <div className="project-detail-page" style={{ maxWidth: '800px', margin: '0 auto', padding: '20px' }}>
      <Link to="/portfolio" className="back-link" style={{ display: 'inline-block', marginBottom: '20px', textDecoration: 'none', color: '#007bff' }}>
        &larr; Back to Portfolio
      </Link>
      
      <div className="project-header" style={{ marginBottom: '40px' }}>
        <h1 style={{ fontSize: '2.5rem', marginBottom: '10px' }}>{project.title}</h1>
        <p className="project-description" style={{ fontSize: '1.2rem', color: '#555', marginBottom: '20px' }}>
          {project.description}
        </p>
        
        <div className="project-meta" style={{ display: 'flex', gap: '20px', marginBottom: '20px', color: '#666' }}>
          {project.role && <p style={{ margin: 0 }}><strong>Role:</strong> {project.role}</p>}
          {project.published_at && (
            <p style={{ margin: 0 }}>
              <strong>Published:</strong> {new Date(project.published_at).toLocaleDateString()}
            </p>
          )}
        </div>

        <div className="project-links" style={{ display: 'flex', gap: '10px' }}>
          {project.live_url && (
            <a href={project.live_url} target="_blank" rel="noopener noreferrer" className="btn-primary" style={{ padding: '8px 16px', backgroundColor: '#007bff', color: 'white', textDecoration: 'none', borderRadius: '4px' }}>
              Live Demo
            </a>
          )}
          {project.repo_url && (
            <a href={project.repo_url} target="_blank" rel="noopener noreferrer" className="btn-secondary" style={{ padding: '8px 16px', backgroundColor: '#6c757d', color: 'white', textDecoration: 'none', borderRadius: '4px' }}>
              View Code
            </a>
          )}
        </div>
      </div>

      <div className="project-content">
        {project.challenge && (
          <section className="project-section" style={{ marginBottom: '30px' }}>
            <h2 style={{ borderBottom: '1px solid #eee', paddingBottom: '10px' }}>The Challenge</h2>
            <p style={{ lineHeight: '1.6' }}>{project.challenge}</p>
          </section>
        )}

        {project.solution && (
          <section className="project-section" style={{ marginBottom: '30px' }}>
            <h2 style={{ borderBottom: '1px solid #eee', paddingBottom: '10px' }}>The Solution</h2>
            <p style={{ lineHeight: '1.6' }}>{project.solution}</p>
          </section>
        )}

        <div className="project-tech-stack" style={{ display: 'flex', gap: '40px', marginBottom: '30px' }}>
          {project.stack && project.stack.length > 0 && (
            <section className="project-section" style={{ flex: 1 }}>
              <h2 style={{ borderBottom: '1px solid #eee', paddingBottom: '10px' }}>Stack</h2>
              <ul className="tag-list" style={{ listStyle: 'none', padding: 0, display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                {project.stack.map((item, idx) => (
                  <li key={idx} className="tag" style={{ backgroundColor: '#f0f0f0', padding: '4px 8px', borderRadius: '4px', fontSize: '0.9rem' }}>
                    {item}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {project.technologies && project.technologies.length > 0 && (
            <section className="project-section" style={{ flex: 1 }}>
              <h2 style={{ borderBottom: '1px solid #eee', paddingBottom: '10px' }}>Technologies</h2>
              <ul className="tag-list" style={{ listStyle: 'none', padding: 0, display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                {project.technologies.map((tech, idx) => (
                  <li key={idx} className="tag" style={{ backgroundColor: '#f0f0f0', padding: '4px 8px', borderRadius: '4px', fontSize: '0.9rem' }}>
                    {tech}
                  </li>
                ))}
              </ul>
            </section>
          )}
        </div>
        
        {project.media_ids && project.media_ids.length > 0 && (
          <section className="project-section project-media" style={{ marginBottom: '30px' }}>
            <h2 style={{ borderBottom: '1px solid #eee', paddingBottom: '10px' }}>Media</h2>
            <div className="media-grid" style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: '16px' }}>
              {project.media_ids.map((media, idx) => (
                <div key={idx} className="media-item" style={{ backgroundColor: '#f9f9f9', padding: '20px', textAlign: 'center', border: '1px solid #ddd', borderRadius: '4px' }}>
                  Media ID: {media}
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </div>
  );
}
