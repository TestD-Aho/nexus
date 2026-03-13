// Public Page View
import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { pages } from '../api';

export function PageViewPage() {
  const { slug } = useParams();
  const [page, setPage] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadPage();
  }, [slug]);

  const loadPage = async () => {
    setLoading(true);
    try {
      const res = await pages.get(slug);
      setPage(res.data);
    } catch (err) {
      setError(err.response?.status === 404 ? 'Page not found' : 'Error loading page');
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <div className="loading">Loading...</div>;
  if (error) return <div className="error-page"><h1>404</h1><p>{error}</p><Link to="/">Go Home</Link></div>;

  const { page: pageData, blocks } = page;

  return (
    <div className="public-page">
      {blocks?.map((block, index) => (
        <BlockRenderer key={block.id || index} block={block} />
      ))}
    </div>
  );
}

function BlockRenderer({ block }) {
  const { block_type, title, content } = block;
  
  switch (block_type) {
    case 'HeroHeader':
      return (
        <section 
          className="block-hero" 
          style={{ backgroundImage: content?.backgroundImage ? `url(${content.backgroundImage})` : undefined }}
        >
          <div className="hero-content">
            <h1>{title || content?.title}</h1>
            <p>{content?.subtitle}</p>
          </div>
        </section>
      );

    case 'RichText':
      return (
        <section className="block-richtext">
          <div dangerouslySetInnerHTML={{ __html: content?.html }} />
        </section>
      );

    case 'ProjectGrid':
      return (
        <section className="block-projects">
          <h2>{title || content?.title || 'Projects'}</h2>
          <div className="projects-grid">
            {(content?.items || []).map((item, i) => (
              <div key={i} className="project-card">
                <h3>{item.title}</h3>
                <p>{item.description}</p>
              </div>
            ))}
          </div>
        </section>
      );

    case 'SkillMatrix':
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

    case 'ContactForm':
      return (
        <section className="block-contact">
          <h2>{title || 'Contact'}</h2>
          <form className="contact-form" onSubmit={e => { e.preventDefault(); alert('Message sent!'); }}>
            <input type="text" placeholder="Name" required />
            <input type="email" placeholder="Email" required />
            <textarea placeholder="Message" required />
            <button type="submit">Send</button>
          </form>
        </section>
      );

    case 'TestimonialSlider':
      return (
        <section className="block-testimonials">
          <h2>{title || 'Testimonials'}</h2>
          <div className="testimonials">
            {(content?.testimonials || []).map((t, i) => (
              <blockquote key={i}>
                <p>"{t.text}"</p>
                <cite>— {t.author}</cite>
              </blockquote>
            ))}
          </div>
        </section>
      );

    default:
      return (
        <section className="block-generic">
          <h3>{title}</h3>
          <pre>{JSON.stringify(content, null, 2)}</pre>
        </section>
      );
  }
}

export default PageViewPage;
