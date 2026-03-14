// Project Card Component
import { useState } from 'react';

export default function ProjectCard({ title, description, tags = [], liveUrl, repoUrl, thumbnail }) {
  const [hovered, setHovered] = useState(false);

  return (
    <div className="project-card" onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}>
      {thumbnail && (
        <div className="project-thumbnail">
          <img src={thumbnail} alt={`${title} thumbnail`} />
        </div>
      )}
      <div className="project-content">
        <h3 className="project-title">{title}</h3>
        <p className="project-description">{description}</p>
        {tags.length > 0 && (
          <div className="project-tags">
            {tags.map((tag, index) => (
              <span key={index} className="tag">{tag}</span>
            ))}
          </div>
        )}
        <div className="project-links">
          {liveUrl && (
            <a href={liveUrl} target="_blank" rel="noopener noreferrer" className="btn-link">
              Live Demo
            </a>
          )}
          {repoUrl && (
            <a href={repoUrl} target="_blank" rel="noopener noreferrer" className="btn-link">
              View Code
            </a>
          )}
        </div>
      </div>
      {hovered && (
        <div className="project-overlay">
          <div className="overlay-content">
            <p>Click to explore</p>
          </div>
        </div>
      )}
    </div>
  );
}