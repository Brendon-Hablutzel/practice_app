import { useEffect, useState } from "react";
import Navbar from "./Navbar";
import { Piece } from "./api-types";
import { fetchPieces, addPiece } from "./fetch";
import styles from "./css/Pieces.module.css";

function Pieces() {
    const [composer, setComposer] = useState("");
    const [title, setTitle] = useState("");
    const [pieces, setPieces] = useState<Piece[]>([]);

    useEffect(() => {
        let searchParams = {
            composer: composer !== "" ? composer : undefined,
            title: title !== "" ? title : undefined,
        };
        if (!(searchParams.composer || searchParams.title)) {
            setPieces([]);
        } else {
            fetchPieces(setPieces, alert, searchParams);
        }
    }, [title, composer]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (composer === "") {
            alert("Invalid composer");
            return;
        }

        if (title === "") {
            alert("Invalid title");
            return;
        }

        addPiece(
            { composer, title },
            () => {
                setComposer("");
                setTitle("");
                fetchPieces(setPieces, alert);
            },
            alert
        );
    };

    return (
        <div>
            <Navbar />
            <h1>Search for or add pieces</h1>
            <form onSubmit={handleSubmit} className={styles.piecesForm}>
                <input
                    type="text"
                    value={composer}
                    onChange={(e) => setComposer(e.target.value)}
                    placeholder="Composer"
                    className={styles.textInput}
                />
                <input
                    type="text"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                    placeholder="Title"
                    className={styles.textInput}
                />
                <input
                    type="submit"
                    value="Add Piece"
                    className={styles.submitButton}
                />
            </form>

            <div className={styles.results}>
                {pieces.map((piece) => {
                    return (
                        <div key={piece.piece_id}>
                            {piece.composer}: {piece.title}
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

export default Pieces;
