import ThemeToggle from "../ThemeToggle";

import classes from './component.module.css';

export interface NavHeaderProps {
    variant?: 'normal' | 'big'
}

const NavHeader: React.FC<NavHeaderProps> = (props) => {
    const {
        variant = 'normal'
    } = props;

    const headerClassName = variant === 'big' ? classes.big : undefined;

    return (
        <header className={classes.container}>
            <h1 className={headerClassName}>🧊&nbsp; Hyprkube </h1>
            <ThemeToggle />
        </header>
    );
};

export default NavHeader;
