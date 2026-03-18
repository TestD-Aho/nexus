import './TestimonialSlider.css';

export function TestimonialSliderView({ block }) {
  const { title, content } = block;
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
}

export function TestimonialSliderEdit({ content, onChange }) {
  return (
    <div className="form-group">
      <label>Testimonials configuration coming soon...</label>
      <p>Edit testimonials here</p>
    </div>
  );
}