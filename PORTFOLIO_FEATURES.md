# Portfolio Models and API

Added:
- Project model (title, slug, description, challenge, solution, stack, role, live_url, repo_url, media_ids, technologies, featured, published_at)
- Skill model (name, category, proficiency, icon)
- Testimonial model (author, author_title, author_company, content, author_image_url, rating)
- BlogPost model (title, slug, content, excerpt, featured_image, published, published_at)
- Projects API with CRUD endpoints
- Updated models/mod.rs with new DTOs
- Updated api/mod.rs to include projects router
- Created projects.rs API handler
