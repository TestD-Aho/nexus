import { HeroHeaderView, HeroHeaderEdit } from './HeroHeader';
import { RichTextView, RichTextEdit } from './RichText';
import { ProjectGridView, ProjectGridEdit } from './ProjectGrid';
import { WorkProcessView, WorkProcessEdit } from './WorkProcess';
import { SkillMatrixView, SkillMatrixEdit } from './SkillMatrix';
import { ContactFormView, ContactFormEdit } from './ContactForm';
import { TestimonialSliderView, TestimonialSliderEdit } from './TestimonialSlider';
import DOMPurify from 'dompurify';

export const BLOCK_REGISTRY = {
  HeroHeader: {
    label: 'Hero Header',
    icon: '🎯',
    defaultContent: { title: 'Welcome', subtitle: 'Your subtitle here', backgroundImage: '' },
    View: HeroHeaderView,
    Edit: HeroHeaderEdit,
    preview: (block) => <div className="preview-hero"><h3>{block.title || block.content?.title}</h3><p>{block.content?.subtitle}</p></div>
  },
  RichText: {
    label: 'Rich Text',
    icon: '📝',
    defaultContent: { html: '<p>Your content here...</p>' },
    View: RichTextView,
    Edit: RichTextEdit,
    preview: (block) => <div className="preview-text" dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(block.content?.html || '') }} />
  },
  ProjectGrid: {
    label: 'Project Grid',
    icon: '🗂️',
    defaultContent: { title: 'My Projects', items: [] },
    View: ProjectGridView,
    Edit: ProjectGridEdit,
    preview: (block) => <div className="preview-grid">{block.content?.items?.length || 0} projects</div>
  },
  WorkProcess: {
    label: 'Work Process',
    icon: '⚙️',
    defaultContent: { steps: [{ title: '', description: '' }] },
    View: WorkProcessView,
    Edit: WorkProcessEdit,
    preview: (block) => <div className="preview-process">{block.content?.steps?.length || 0} steps</div>
  },
  SkillMatrix: {
    label: 'Skill Matrix',
    icon: '⚡',
    defaultContent: { skills: [] },
    View: SkillMatrixView,
    Edit: SkillMatrixEdit,
    preview: (block) => <div className="preview-skills">{(block.content?.skills || []).join(', ')}</div>
  },
  ContactForm: {
    label: 'Contact Form',
    icon: '📧',
    defaultContent: { fields: ['name', 'email', 'message'], recipients: [] },
    View: ContactFormView,
    Edit: ContactFormEdit,
    preview: () => <div className="preview-generic">Contact Form</div>
  },
  TestimonialSlider: {
    label: 'Testimonials',
    icon: '💬',
    defaultContent: { testimonials: [] },
    View: TestimonialSliderView,
    Edit: TestimonialSliderEdit,
    preview: (block) => <div className="preview-generic">{block.content?.testimonials?.length || 0} testimonials</div>
  }
};