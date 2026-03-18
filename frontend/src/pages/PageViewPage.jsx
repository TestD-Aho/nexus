// Public Page View
import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { pages } from '../api';
import { BLOCK_REGISTRY } from '../components/blocks/registry';
import { motion } from 'framer-motion';

export function PageViewPage() {
  const { slug } = useParams();
  const [page, setPage] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadPage();
  }, [slug]);

  // Dynamic SEO meta tags
  useEffect(() => {
    if (page?.page) {
      const { page: pageData } = page;
      document.title = pageData.meta_title || `${pageData.title} | Nexus CMS`;
      
      // Update meta description
      let metaDesc = document.querySelector('meta[name="description"]');
      if (!metaDesc) {
        metaDesc = document.createElement('meta');
        metaDesc.name = 'description';
        document.head.appendChild(metaDesc);
      }
      metaDesc.setAttribute('content', pageData.meta_description || pageData.description || '');
    }
  }, [page]);

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
  const registryEntry = BLOCK_REGISTRY[block.block_type];
  const ViewComponent = registryEntry?.View;

  return (
    <motion.div 
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-50px" }}
      transition={{ duration: 0.6, ease: "easeOut" }}
    >
      {ViewComponent ? (
        <ViewComponent block={block} />
      ) : (
        <section className="block-generic">
          <h3>{block.title}</h3>
          <pre>{JSON.stringify(block.content, null, 2)}</pre>
        </section>
      )}
    </motion.div>
  );
}

export default PageViewPage;