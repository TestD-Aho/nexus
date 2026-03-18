// Project Card Component
import { useState } from 'react';
import { motion } from 'framer-motion';
import { Link } from 'react-router-dom';

export default function ProjectCard({ project }) {
  if (!project) return null;
  const { slug, title, description, stack = [], live_url, repo_url, thumbnail } = project;

  return (
    <motion.div 
      className="project-card"
      whileHover={{ y: -5 }}
      transition={{ type: "spring", stiffness: 300 }}
      style={{
        position: 'relative',
        overflow: 'hidden',
        background: 'var(--bg-card)',
        padding: 'var(--space-3)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--border)',
        cursor: 'pointer'
      }}
    >
      <motion.div
        className="project-overlay"
        initial={{ opacity: 0 }}
        whileHover={{ opacity: 1 }}
        style={{
          position: 'absolute',
          inset: 0,
          background: 'rgba(255, 95, 31, 0.9)',
          zIndex: 10,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#000',
          fontWeight: 'bold',
          fontSize: '1.2rem'
        }}
      >
        <Link to={`/project/${slug}`} style={{ color: '#000', textDecoration: 'none' }}>Explore Case Study &rarr;</Link>
      </motion.div>

      {thumbnail && (
        <motion.div 
          className="project-thumbnail"
          whileHover={{ scale: 1.05 }}
          style={{ marginBottom: 'var(--space-2)' }}
        >
          <img src={thumbnail} alt={`${title} thumbnail`} style={{ width: '100%', height: 'auto', borderRadius: 'var(--radius-sm)' }} />
        </motion.div>
      )}
      
      <div className="project-content" style={{ position: 'relative', zIndex: 1 }}>
        <h3 className="project-title" style={{ fontFamily: 'var(--font-heading)', fontSize: '1.8rem', marginBottom: 'var(--space-1)' }}>{title}</h3>
        <p className="project-description" style={{ opacity: 0.8, marginBottom: 'var(--space-2)' }}>{description}</p>
        
        {stack?.length > 0 && (
          <div className="project-tags" style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: 'var(--space-3)' }}>
            {stack.map((tag, index) => (
              <span key={index} className="tag" style={{ background: 'var(--border)', padding: '4px 12px', borderRadius: '20px', fontSize: '0.8rem' }}>{tag}</span>
            ))}
          </div>
        )}
        
        <div className="project-links" style={{ display: 'flex', gap: '16px' }}>
          <Link to={`/project/${slug}`} className="btn-link" style={{ color: 'var(--primary)', fontWeight: 'bold', textDecoration: 'none' }}>Details</Link>
          {live_url && (
            <a href={live_url} target="_blank" rel="noopener noreferrer" className="btn-link" style={{ color: 'var(--text)', textDecoration: 'none' }}>Live Demo</a>
          )}
          {repo_url && (
            <a href={repo_url} target="_blank" rel="noopener noreferrer" className="btn-link" style={{ color: 'var(--text)', textDecoration: 'none' }}>GitHub</a>
          )}
        </div>
      </div>
    </motion.div>
  );
}