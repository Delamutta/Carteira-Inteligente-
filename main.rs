```rust
use axum::{
    extract::Json,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Asset {
    id: u32,
    symbol: String,
    name: String,
    quantity: f64,
    average_price: f64,
    current_price: f64,
}

impl Asset {
    fn invested_value(&self) -> f64 {
        self.quantity * self.average_price
    }

    fn total_value(&self) -> f64 {
        self.quantity * self.current_price
    }

    fn profit_loss(&self) -> f64 {
        self.total_value() - self.invested_value()
    }

    fn profit_loss_percentage(&self) -> f64 {
        if self.invested_value() == 0.0 {
            return 0.0;
        }

        (self.profit_loss() / self.invested_value()) * 100.0
    }
}

#[derive(Clone)]
struct AppState {
    assets: Arc<Mutex<Vec<Asset>>>,
}

#[derive(Debug, Deserialize)]
struct CreateAsset {
    symbol: String,
    name: String,
    quantity: f64,
    average_price: f64,
    current_price: f64,
}

#[derive(Debug, Serialize)]
struct PortfolioSummary {
    total_invested: f64,
    total_value: f64,
    total_profit_loss: f64,
    profit_loss_percentage: f64,
    risk_level: String,
}

#[tokio::main]
async fn main() {
    let assets = vec![
        Asset {
            id: 1,
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            quantity: 0.1,
            average_price: 60000.0,
            current_price: 68000.0,
        },
        Asset {
            id: 2,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            quantity: 1.5,
            average_price: 2800.0,
            current_price: 3200.0,
        },
    ];

    let state = AppState {
        assets: Arc::new(Mutex::new(assets)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/assets", get(get_assets))
        .route("/api/assets", post(create_asset))
        .route("/api/summary", get(get_summary))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Não foi possível iniciar o servidor");

    println!("Servidor iniciado em:");
    println!("http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Erro no servidor");
}

async fn index() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html lang="pt-BR">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">

    <title>Crypto Portfolio</title>

    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>

    <style>

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
            font-family: Arial, sans-serif;
        }

        body {
            background: #0f172a;
            color: white;
            padding: 30px;
        }

        .container {
            max-width: 1200px;
            margin: auto;
        }

        h1 {
            margin-bottom: 30px;
        }

        .summary {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 20px;
            margin-bottom: 30px;
        }

        .card {
            background: #1e293b;
            padding: 25px;
            border-radius: 15px;
        }

        .card h3 {
            color: #94a3b8;
            margin-bottom: 10px;
        }

        .card strong {
            font-size: 24px;
        }

        .positive {
            color: #22c55e;
        }

        .negative {
            color: #ef4444;
        }

        .content {
            display: grid;
            grid-template-columns: 2fr 1fr;
            gap: 20px;
        }

        .panel {
            background: #1e293b;
            padding: 25px;
            border-radius: 15px;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }

        th,
        td {
            padding: 15px;
            text-align: left;
            border-bottom: 1px solid #334155;
        }

        th {
            color: #94a3b8;
        }

        input {
            width: 100%;
            padding: 12px;
            margin: 8px 0;
            border-radius: 8px;
            border: none;
            background: #334155;
            color: white;
        }

        button {
            width: 100%;
            padding: 12px;
            margin-top: 10px;
            border: none;
            border-radius: 8px;
            background: #2563eb;
            color: white;
            cursor: pointer;
            font-weight: bold;
        }

        button:hover {
            background: #1d4ed8;
        }

        @media (max-width: 900px) {

            .summary {
                grid-template-columns: repeat(2, 1fr);
            }

            .content {
                grid-template-columns: 1fr;
            }
        }

    </style>
</head>

<body>

<div class="container">

    <h1>Crypto Portfolio</h1>

    <div class="summary">

        <div class="card">
            <h3>Valor Investido</h3>
            <strong id="invested">$0.00</strong>
        </div>

        <div class="card">
            <h3>Valor Atual</h3>
            <strong id="total">$0.00</strong>
        </div>

        <div class="card">
            <h3>Lucro / Prejuízo</h3>
            <strong id="profit">$0.00</strong>
        </div>

        <div class="card">
            <h3>Nível de Risco</h3>
            <strong id="risk">-</strong>
        </div>

    </div>

    <div class="content">

        <div class="panel">

            <h2>Minha Carteira</h2>

            <table>

                <thead>
                    <tr>
                        <th>Ativo</th>
                        <th>Quantidade</th>
                        <th>Preço Atual</th>
                        <th>Valor</th>
                        <th>Resultado</th>
                    </tr>
                </thead>

                <tbody id="assets"></tbody>

            </table>

        </div>

        <div class="panel">

            <h2>Adicionar Ativo</h2>

            <input id="symbol" placeholder="Símbolo: BTC">

            <input id="name" placeholder="Nome: Bitcoin">

            <input id="quantity" type="number" placeholder="Quantidade">

            <input id="average_price" type="number" placeholder="Preço médio">

            <input id="current_price" type="number" placeholder="Preço atual">

            <button onclick="addAsset()">
                Adicionar Ativo
            </button>

        </div>

    </div>

</div>

<script>

async function loadAssets() {

    const response = await fetch('/api/assets');

    const assets = await response.json();

    const table = document.getElementById('assets');

    table.innerHTML = '';

    assets.forEach(asset => {

        const value =
            asset.quantity * asset.current_price;

        const invested =
            asset.quantity * asset.average_price;

        const profit =
            value - invested;

        const color =
            profit >= 0
                ? 'positive'
                : 'negative';

        table.innerHTML += `

            <tr>

                <td>
                    <strong>${asset.symbol}</strong>
                    <br>
                    ${asset.name}
                </td>

                <td>
                    ${asset.quantity}
                </td>

                <td>
                    $${asset.current_price.toFixed(2)}
                </td>

                <td>
                    $${value.toFixed(2)}
                </td>

                <td class="${color}">
                    $${profit.toFixed(2)}
                </td>

            </tr>

        `;

    });

}

async function loadSummary() {

    const response =
        await fetch('/api/summary');

    const summary =
        await response.json();

    document.getElementById('invested')
        .innerText =
        '$' + summary.total_invested.toFixed(2);

    document.getElementById('total')
        .innerText =
        '$' + summary.total_value.toFixed(2);

    const profit =
        document.getElementById('profit');

    profit.innerText =
        '$' + summary.total_profit_loss.toFixed(2);

    profit.className =
        summary.total_profit_loss >= 0
            ? 'positive'
            : 'negative';

    document.getElementById('risk')
        .innerText =
        summary.risk_level;

}

async function addAsset() {

    const asset = {

        symbol:
            document.getElementById('symbol').value,

        name:
            document.getElementById('name').value,

        quantity:
            Number(
                document.getElementById('quantity').value
            ),

        average_price:
            Number(
                document.getElementById('average_price').value
            ),

        current_price:
            Number(
                document.getElementById('current_price').value
            )

    };

    await fetch('/api/assets', {

        method: 'POST',

        headers: {
            'Content-Type':
                'application/json'
        },

        body:
            JSON.stringify(asset)

    });

    await loadAssets();

    await loadSummary();

}

loadAssets();

loadSummary();

</script>

</body>

</html>
"#,
    )
}

async fn get_assets(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<Vec<Asset>> {
    let assets = state.assets.lock().unwrap();

    Json(assets.clone())
}

async fn create_asset(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(input): Json<CreateAsset>,
) -> (StatusCode, Json<Asset>) {
    let mut assets = state.assets.lock().unwrap();

    let id = assets.len() as u32 + 1;

    let asset = Asset {
        id,
        symbol: input.symbol,
        name: input.name,
        quantity: input.quantity,
        average_price: input.average_price,
        current_price: input.current_price,
    };

    assets.push(asset.clone());

    (StatusCode::CREATED, Json(asset))
}

async fn get_summary(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<PortfolioSummary> {
    let assets = state.assets.lock().unwrap();

    let total_invested: f64 =
        assets.iter()
            .map(|asset| asset.invested_value())
            .sum();

    let total_value: f64 =
        assets.iter()
            .map(|asset| asset.total_value())
            .sum();

    let total_profit_loss =
        total_value - total_invested;

    let profit_loss_percentage =
        if total_invested == 0.0 {
            0.0
        } else {
            (total_profit_loss / total_invested) * 100.0
        };

    let risk_level =
        if assets.len() >= 5 {
            "Moderado"
        } else {
            "Alto"
        };

    Json(PortfolioSummary {
        total_invested,
        total_value,
        total_profit_loss,
        profit_loss_percentage,
        risk_level: risk_level.to_string(),
    })
}
```
