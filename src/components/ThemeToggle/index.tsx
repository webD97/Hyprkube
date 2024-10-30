const ThemeToggle: React.FC = () => {
    return (
        <button onClick={() => {
            document.querySelector(':root')?.classList.toggle('dark-mode');
        }}>💡</button>
    );
};

export default ThemeToggle;
