var host = "http://45.55.75.16:80";

function parseID(id) {
    return parseInt(id.substring(8));
}

function pollForDomNodes(query, init) {
    var nodes = [];
    function poll() {
        var newNodes = query().filter(function(newNode) {
            return nodes.indexOf(newNode) === -1;
        });

        for (var newNode of newNodes) {
            nodes.push(newNode);
            init($(newNode));
        }
    }
    poll();
    window.setInterval(poll, 1000);
}

function removedMessageShower() {
    function getMessage(messageID, callback) {
        return $.ajax({
            type: "GET",
            url: host + "/message",
            data: {message_id: messageID},
            success: callback,
        });
    }

    function showDeletedMessage(message) {
        var myMessageID = parseID(message.parents('.message').attr('id'));
        getMessage(myMessageID, function(content){
            if (content != 'No message') {
                message.text(content);
                message.removeClass('deleted');
            }
        });
    }

    function lookForMoreDeletedMessages() {
        return Array.from($("span.deleted"));
    }

    pollForDomNodes(lookForMoreDeletedMessages, showDeletedMessage);
}

$(function(){
    removedMessageShower();
});
