import './ContactForm.css';

export function ContactFormView({ block }) {
  const { title } = block;
  return (
    <section className="block-contact">
      <h2>{title || 'Contact'}</h2>
      <form className="contact-form" onSubmit={e => { e.preventDefault(); alert('Message sent!'); }}>
        <input type="text" placeholder="Name" required />
        <input type="email" placeholder="Email" required />
        <textarea placeholder="Message" required />
        <button type="submit" className="btn-primary">Send</button>
      </form>
    </section>
  );
}

export function ContactFormEdit({ content, onChange }) {
  return (
    <div className="form-group">
      <label>Form configuration coming soon...</label>
      <p>Edit form settings here</p>
    </div>
  );
}