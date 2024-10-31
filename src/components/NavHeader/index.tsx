import { useLocation, useNavigate } from "react-router-dom";
import ThemeToggle from "../ThemeToggle";

import classes from './component.module.css';
import { useEffect, useState } from "react";

export interface NavHeaderProps {
}

const NavHeader: React.FC<NavHeaderProps> = (_props) => {
    const location = useLocation();
    const navigate = useNavigate();

    const [canGoBack, setCanGoBack] = useState(false);

    // Disable the back button if we cannot go back further
    useEffect(() => {
        setCanGoBack(location.key !== "default");
    }, [location]);

    return (
        <div className={classes.container}>
            <button disabled={!canGoBack} onClick={() => navigate(-1)}>&larr;</button>
            <button disabled={!canGoBack} onClick={() => navigate(1)}>&rarr;</button>
            <h2>🧊&nbsp; Hyprkube </h2>
            <div>
                <ThemeToggle />
            </div>
        </div>
    );
};

export default NavHeader;
