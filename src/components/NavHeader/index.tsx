import ThemeToggle from "../ThemeToggle";

import classes from './component.module.css';

const NavHeader: React.FC = () => {
    return (
        <header className={classes.container}>
            <h1>🧊&nbsp; Hyprkube </h1>
            <ThemeToggle />
        </header>
    );
};

export default NavHeader;
