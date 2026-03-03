// vex Documentation Custom JavaScript

// Smooth scroll behavior
document.addEventListener('DOMContentLoaded', function() {
  // Smooth scrolling for anchor links
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function(e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute('href'));
      if (target) {
        target.scrollIntoView({ behavior: 'smooth' });
      }
    });
  });

  // Add copy button to code blocks
  document.querySelectorAll('pre > code').forEach(code => {
    const pre = code.parentElement;
    if (!pre.querySelector('.copy-button')) {
      const button = document.createElement('button');
      button.className = 'copy-button';
      button.textContent = 'Copy';
      button.style.cssText = `
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        padding: 0.5rem 1rem;
        background-color: rgba(255, 255, 255, 0.1);
        color: #e5e7eb;
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 0.375rem;
        cursor: pointer;
        font-size: 0.875rem;
        transition: all 0.3s ease;
        z-index: 10;
      `;

      button.addEventListener('mouseover', () => {
        button.style.backgroundColor = 'rgba(255, 255, 255, 0.2)';
      });

      button.addEventListener('mouseout', () => {
        button.style.backgroundColor = 'rgba(255, 255, 255, 0.1)';
      });

      button.addEventListener('click', () => {
        navigator.clipboard.writeText(code.textContent);
        button.textContent = 'Copied!';
        setTimeout(() => {
          button.textContent = 'Copy';
        }, 2000);
      });

      pre.style.position = 'relative';
      pre.insertBefore(button, code);
    }
  });

  // Add external link indicators
  document.querySelectorAll('a[href^="http"]').forEach(link => {
    if (!link.hostname || link.hostname !== window.location.hostname) {
      link.setAttribute('target', '_blank');
      link.setAttribute('rel', 'noopener noreferrer');
      const icon = document.createElement('span');
      icon.textContent = ' ↗';
      icon.style.fontSize = '0.85em';
      link.appendChild(icon);
    }
  });

  // Highlight active sections in table of contents
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const id = entry.target.getAttribute('id');
        document.querySelectorAll('.md-nav__link').forEach(link => {
          link.classList.remove('active');
          if (link.getAttribute('href') === '#' + id) {
            link.classList.add('active');
          }
        });
      }
    });
  }, { threshold: 0.5 });

  document.querySelectorAll('h2, h3').forEach(heading => {
    observer.observe(heading);
  });

  // Performance: Add lazy loading to images
  document.querySelectorAll('img').forEach(img => {
    if (!img.hasAttribute('loading')) {
      img.setAttribute('loading', 'lazy');
    }
  });
});

// Page view tracking (optional)
if (window.gtag) {
  document.addEventListener('page', () => {
    gtag('config', 'G_MEASUREMENT_ID');
  });
}
