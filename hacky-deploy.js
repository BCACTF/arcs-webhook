const { request } = require('http');

const deployToken = process.env.DEPLOY_AUTH_TOKEN;
const address = process.env.WEBHOOK_ADDRESS;

const challsToDeploy = process.argv.slice(2).flatMap(s => s.split(','));
console.log(`Deploying: ${challsToDeploy.join(', ')}`);




const deployStates = {

};
const deployIds = {

};

for (const chall of challsToDeploy) deployStates[chall] = "Not started";


const sendInitial = async() => {
    for (const chall of challsToDeploy) {
        const res = await sendReq({
            "deploy": {
                "__type": "deploy",
                "chall": chall,
                "force_wipe": false
            }
        });

        // deployStates[chall] = res.deploy.status;
        deployIds[chall] = res.deploy.poll_id;
    }
};

sendInitial();


setInterval(async () => {
    for (const chall of challsToDeploy) {
        const res = await sendReq({
            deploy: {
                __type: "poll",
                id: deployIds[chall]
            }
        });

        try {
            const name = res.deploy.status.padStart(9, " ");
            const time = (res.deploy.status_time.secs + res.deploy.status_time.nanos * 1e-9);
            deployStates[chall] = `${name} for ${time.toFixed(1)} seconds`;
            deployIds[chall] = res.deploy.poll_id;
        } catch (e) {
            console.error(e);
            console.log(res.toString());
            console.log(res);
        }
    }

    console.log('\n\nUpdate:');
    challsToDeploy.forEach(chall => console.log(`${deployIds[chall]} ${chall.padStart(16, " ")}: ${deployStates[chall]}`))
    console.log();
    // Promise.allSettled(promises).then(() => {
    // });
}, 5000);




function sendReq(json) {
    return new Promise((resolve, rej) => {
        const req = request(
            address,
            { method: "POST", headers: {
                "Authorization": `Bearer ${deployToken}`,
                "Content-Type": "application/json",
            } },
            res => {
                const chunks = [];
                res.on('data', data => chunks.push(data))
                res.on('end', () => {
                    let resBody = Buffer.concat(chunks);
                    switch(res.headers['content-type']) {
                        case 'application/json':
                            resBody = JSON.parse(resBody);
                            break;
                    }
                    resolve(resBody)
                })
            }
        );
        
        req.on('error', rej);
        req.write(JSON.stringify(json));
        req.end();
    })
}
