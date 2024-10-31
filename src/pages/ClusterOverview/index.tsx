import { Link } from "react-router-dom";
import NavHeader from "../../components/NavHeader";

import classes from './styles.module.css';

const ClusterOverview: React.FC = () => {
    return (
        <div className={classes.container}>
            <div>
                <NavHeader variant="big" />
                <Link to="/cluster">Go to cluster</Link>
            </div>
        </div>
    );
};

export default ClusterOverview;
