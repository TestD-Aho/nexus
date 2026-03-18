import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { collections } from '../api';

export default function BlogPostPage() {
  const { id } = useParams();
  const [post, setPost] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const loadPost = async () => {
      try {
        setLoading(true);
        const res = await collections.get('blog');
        const items = res.data?.items || res.data || [];
        const itemsArray = Array.isArray(items) ? items : [];
        const foundPost = itemsArray.find(item => String(item.id) === String(id) || item.slug === id);
        
        if (foundPost) {
          setPost(foundPost);
        } else {
          setError('Post not found');
        }
      } catch (err) {
        console.error('Failed to load post:', err);
        setError('Could not load post.');
      } finally {
        setLoading(false);
      }
    };
    loadPost();
  }, [id]);

  if (loading) return <div className="loading" style={{ textAlign: 'center', padding: '40px' }}>Loading post...</div>;
  if (error || !post) return <div className="error" style={{ textAlign: 'center', color: 'red', padding: '40px' }}>{error || 'Post not found'}</div>;

  return (
    <div className="blog-post-page" style={{ maxWidth: '800px', margin: '0 auto', padding: '20px' }}>
      <Link to="/blog" className="back-link" style={{ display: 'inline-block', marginBottom: '30px', textDecoration: 'none', color: '#007bff' }}>
        &larr; Back to Blog
      </Link>
      
      <article className="blog-post">
        <header className="post-header" style={{ marginBottom: '40px', textAlign: 'center' }}>
          <h1 style={{ fontSize: '3rem', marginBottom: '15px' }}>{post.title || post.name}</h1>
          <div className="post-meta" style={{ display: 'flex', justifyContent: 'center', gap: '15px', color: '#666', fontSize: '1.1rem' }}>
            {post.author && <span className="author">By <strong>{post.author}</strong></span>}
            {post.published_at && <span className="date">{new Date(post.published_at).toLocaleDateString()}</span>}
          </div>
        </header>
        
        <div className="post-content" style={{ lineHeight: '1.8', fontSize: '1.1rem', color: '#333' }}>
          {post.content ? (
            <div dangerouslySetInnerHTML={{ __html: post.content }} />
          ) : (
             <p>{post.body || post.description}</p>
          )}
        </div>
      </article>
    </div>
  );
}
