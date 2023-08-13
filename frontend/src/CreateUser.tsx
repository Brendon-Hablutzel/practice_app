import { useContext, useState } from "react";
import { AuthContext } from "./Auth";
import { useLocation, useNavigate } from "react-router-dom";
import Navbar from "./Navbar";
import CredentialForm from "./CredentialForm";

function CreateUser() {
    const [userName, setUserName] = useState("");
    const [password, setPassword] = useState("");
    const auth = useContext(AuthContext);
    const navigate = useNavigate();
    const location = useLocation();

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (userName === "") {
            alert("Invalid username");
            return;
        }

        if (password === "") {
            alert("Invalid password");
            return;
        }

        auth.createUser(userName, password, (success) => {
            if (success) {
                auth.login(userName, password, (success) => {
                    if (success) {
                        navigate("/", {
                            replace: true,
                            state: { from: location },
                        });
                    } else {
                        alert("Failed to log in after user creation");
                    }
                });
            } else {
                alert("Unable to create account");
            }
        });
    };

    if (auth.getUser()) {
        return <div>You are already logged in</div>;
    }

    return (
        <div>
            <Navbar />
            <h1>Create User</h1>
            <CredentialForm
                handleSubmit={handleSubmit}
                userName={userName}
                setUserName={setUserName}
                password={password}
                setPassword={setPassword}
            />
        </div>
    );
}

export default CreateUser;
