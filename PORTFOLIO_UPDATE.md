# Portfolio Models and API

Added portfolio functionality to Nexus CMS:

## New Models
- **Project**: Portfolio projects with title, slug, description, challenge, solution, stack, role, live_url, repo_url, media_ids, technologies, featured, published_at
- **Skill**: Technical skills with name, category, proficiency (0-100), icon
- **Testimonial**: Client testimonials with author, title, company, content, image, rating
- **BlogPost**: Blog posts with title, slug, content, excerpt, featured_image, published, published_at

## New API Endpoints
- GET /projects - List projects (with featured filter)
- GET /projects/:slug - Get project by slug
- POST /projects - Create project (protected)
- PUT /projects/:id - Update project (protected)
- DELETE /projects/:id - Delete project (protected)

## Files Modified
- backend/src/models/mod.rs - Added Project, Skill, Testimonial, BlogPost models and DTOs
- backend/src/api/projects.rs - New API handler for projects
- backend/src/api/mod.rs - Added projects to the router

## Features
- Project filtering (featured/all)
- Media associations (media_ids array)
- Technology stack storage (technologies array)
- Featured project flag
- Live/demo and repository URLs
- Role description (what you did on the project)
- Challenge/solution description for case studies

Ready for frontend portfolio section implementation.