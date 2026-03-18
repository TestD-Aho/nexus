import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { collections } from '../api';

export default function BlogPage() {
  const [posts, setPosts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const loadPosts = async () => {
      try {
        setLoading(true);
        const res = await collections.get('blog');
        const items = res.data?.items || res.data || [];
        setPosts(Array.isArray(items) ? items : []);
      } catch (err) {
        console.error('Failed to load blog posts:', err);
        setError('Could not load blog posts.');
      } finally {
        setLoading(false);
      }
    };
    loadPosts();
  }, []);

  if (loading) return <div className="loading" style={{ textAlign: 'center', padding: '40px' }}>Loading blog...</div>;
  if (error) return <div className="error" style={{ textAlign: 'center', color: 'red', padding: '40px' }}>{error}</div>;

  return (
    <div className="blog-page" style={{ maxWidth: '800px', margin: '0 auto', padding: '20px' }}>
      <div className="page-header" style={{ marginBottom: '40px' }}>
        <h1 style={{ fontSize: '2.5rem' }}>Blog</h1>
      </div>
      <div className="blog-grid" style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
        {posts.length === 0 ? (
          <p style={{ textAlign: 'center', color: '#666' }}>No blog posts found.</p>
        ) : (
          posts.map(post => (
            <div key={post.id} className="blog-card" style={{ border: '1px solid #eaeaea', borderRadius: '8px', padding: '24px', backgroundColor: '#fff', boxShadow: '0 2px 4px rgba(0,0,0,0.05)' }}>
              <h2 style={{ marginTop: 0, marginBottom: '10px' }}>
                <Link to={`/blog/${post.slug || post.id}`} style={{ textDecoration: 'none', color: '#333' }}>
                  {post.title || post.name}
                </Link>
              </h2>
              {post.excerpt && <p style={{ color: '#555', lineHeight: '1.5', marginBottom: '15px' }}>{post.excerpt}</p>}
              <div className="blog-meta" style={{ fontSize: '0.9rem', color: '#888' }}>
                {post.published_at && <span>{new Date(post.published_at).toLocaleDateString()}</span>}
                {post.author && <span style={{ marginLeft: '10px' }}>&bull; {post.author}</span>}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
