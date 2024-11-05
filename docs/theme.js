document.getElementById('theme-toggle').addEventListener('change', function() {
    document.documentElement.classList.toggle('dark-mode', this.checked);
    // Optional: Persist theme preference in localStorage
    localStorage.setItem('theme', this.checked ? 'dark' : 'light');
});

// Check for saved theme preference or system preference on page load
document.addEventListener('DOMContentLoaded', () => {
    const savedTheme = localStorage.getItem('theme');
    const themeToggle = document.getElementById('theme-toggle');

    if (savedTheme === 'dark') {
        document.documentElement.classList.add('dark-mode');
        themeToggle.checked = true;
    }
});
