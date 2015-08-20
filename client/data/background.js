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

function starIdentifier() {
    function getStars(messageID, callback) {
        return $.ajax({
            type: "GET",
            url: host + "/stars",
            data: {message_id: messageID},
            success: function(value){callback(JSON.parse(value));},
        });
    }

    function getStarsFromDOM(star, box) {
        box.html(''); // Clear the box out
        var myMessageID = parseID(star.parents('li, .message').attr('id'));
        return getStars(myMessageID, function(stars){
            box.append('<h4> Starred by: </h4>');
            var innerList = $('<ul>');
            var names = stars.map(function(star){
                return star.user_name;
            });
            for (var name of names) {
                var listItem = $('<li>');
                listItem.text(name);
                innerList.append(listItem);
            }
            box.append(innerList);
        });
    }

    function setUpStar(star) {
        var box = $('<div class="blah"/>');
        star.append(box);
        var currentRequest = getStarsFromDOM(star, box);
        star.parent().mouseenter(function(){
            currentRequest.abort();
            currentRequest = getStarsFromDOM(star, box);
            box.show();
        });
        star.parent().mouseleave(function(){
            box.hide();
        });
    }

    function lookForMoreStars() {
        return Array.from($("span.stars:visible")).filter(function(newStar){
            return $(newStar).parent("span.meta").length == 0;
        });
    }

    pollForDomNodes(lookForMoreStars, setUpStar);
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
    starIdentifier();
    removedMessageShower();
});