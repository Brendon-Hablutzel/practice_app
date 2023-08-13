import { useContext, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { AuthContext } from "./Auth";

function Logout() {
    const navigate = useNavigate();
    const auth = useContext(AuthContext);
    const [message, setMessage] = useState("Logging out...");

    useEffect(() => {
        if (auth.getUser()) {
            auth.logout((logoutSuccess) => {
                if (logoutSuccess) {
                    setMessage(
                        "Successfully logged out, you may close this page"
                    );
                } else {
                    setMessage("Unable to log out");
                }
            });
        } else {
            setMessage("You are not logged in");
        }
    }, [navigate, auth]);

    return <div>{message}</div>;
}

export default Logout;
